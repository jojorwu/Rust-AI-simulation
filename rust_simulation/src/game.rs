use super::map::{Map, Tile};
use super::player::Player;
use super::brain::{Brain, HighLevelState};
use super::recipes::RecipeManager;
use super::errors::SimulationError;
use super::item::ItemRegistry;
use super::ecs::{World, Entity};
use super::components::Position;
use super::events::EventBus;
use super::systems::{movement_system, gathering_system, crafting_system, building_system, combat_system, pickup_system, death_system};
use std::sync::{Arc, Mutex};
use tokio::task;

use rand::Rng;

use super::config::*;


/// The main struct for the simulation.
/// It holds the game state, including the map, the ECS world, and the brains for the agents.
pub struct Game {
    pub map: Map,
    pub world: Arc<Mutex<World>>,
    pub brains: Vec<Arc<Mutex<Brain>>>,
    pub item_registry: ItemRegistry,
    pub recipe_manager: Arc<RecipeManager>,
    pub event_bus: Arc<Mutex<EventBus>>,
    pub tick_count: u32,
}


impl Game {
    pub fn new(
        biomes_path: &str,
        resources_path: &str,
        items_path: &str,
        recipes_path: &str,
    ) -> Self {
        let map = Map::new(WIDTH, HEIGHT, biomes_path, resources_path).expect("Failed to create map");
        let item_registry = ItemRegistry::new(items_path);
        let recipe_manager = Arc::new(RecipeManager::new(recipes_path));
        let event_bus = Arc::new(Mutex::new(EventBus::new()));

        let mut world = World::new();
        let mut brains = Vec::new();

        for i in 0..NUM_PLAYERS {
            let player = world.create_entity();
            world.add_component(player, Player::new(i as u32));
            world.add_component(player, Position { x: 0, y: 0 });
            world.add_component(player, crate::components::Health { current: 100, max: 100 });
            brains.push(Arc::new(Mutex::new(Brain::new(Arc::clone(&recipe_manager), 0.1, 0.9, 1.0))));
        }

        Game {
            map,
            world: Arc::new(Mutex::new(world)),
            brains,
            item_registry,
            recipe_manager,
            event_bus,
            tick_count: 0,
        }
    }

    fn find_and_set_valid_start_positions(&mut self) {
        let mut rng = rand::thread_rng();
        let mut occupied_positions = std::collections::HashSet::new();

        let plains_biome = self.map.biomes.iter().find(|b| b.name == "plains");
        let plains_tile_type = plains_biome.map_or('.', |b| b.tile_type);

        let mut world = self.world.lock().expect("Failed to lock world");
        for entity in 0..world.entities.len() {
            loop {
                let x = rng.gen_range(0..self.map.width);
                let y = rng.gen_range(0..self.map.height);
                if self.map.grid[y as usize][x as usize].tile_type == plains_tile_type && !occupied_positions.contains(&(x, y)) {
                    if let Some(pos) = world.get_component_mut::<Position>(entity) {
                        pos.x = x;
                        pos.y = y;
                    }
                    occupied_positions.insert((x, y));
                    break;
                }
            }
        }
    }

    fn setup_new_map(&mut self) {
        self.map.generate_island_map(25.0, 5, 0.5, 2.0);
        self.place_resources();
    }

    fn place_resources(&mut self) {
        let mut rng = rand::thread_rng();
        let mut world = self.world.lock().expect("Failed to lock world");
        self.map.spatial_map.clear();

        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let tile = &self.map.grid[y as usize][x as usize];
                for resource_def in &self.map.resources {
                    if resource_def.biomes.contains(&tile.biome) {
                        if rng.r#gen::<f64>() < resource_def.density {
                            let resource_entity = world.create_entity();
                            world.add_component(resource_entity, Position { x, y });
                            world.add_component(resource_entity, crate::components::Resource {
                                name: resource_def.name.clone(),
                                quantity: 5, // Placeholder quantity
                            });
                            self.map.spatial_map.entry((x, y)).or_default().push(resource_entity);
                            break;
                        }
                    }
                }
            }
        }
    }

    fn get_high_level_state(&self, entity: Entity) -> Result<HighLevelState, SimulationError> {
        let world = self.world.lock().expect("Failed to lock world");
        let player = world.get_component::<Player>(entity)
            .ok_or_else(|| SimulationError::ComponentNotFound("Player".to_string()))?;
        let health = world.get_component::<crate::components::Health>(entity)
            .ok_or_else(|| SimulationError::ComponentNotFound("Health".to_string()))?;
        let brain_lock = self.brains[entity].lock().expect("Failed to lock brain");

        let num_hostile_players = brain_lock.player_memories.values().filter(|m| m.relationship == super::brain::RelationshipStatus::Hostile).count() as u32;

        Ok(HighLevelState {
            has_wood: player.get_total_quantity("wood") > 0,
            has_stone: player.get_total_quantity("stone") > 0,
            has_iron_ore: player.get_total_quantity("iron_ore") > 0,
            has_stone_axe: player.get_total_quantity("stone_axe") > 0,
            num_hostile_players,
            health_level: health.current as u32,
        })
    }

    pub async fn run(&mut self) -> Result<(), SimulationError> {
        println!("--- Starting Rust Training Simulation ---");
        self.setup_new_map();
        self.find_and_set_valid_start_positions();

        println!("Initial Map:");
        self.map.display(&self.world.lock().expect("Failed to lock world"));

        for episode in 0..EPISODES {
            self.run_episode(episode).await?;
            if (episode + 1) % 200 == 0 {
                println!("Episode {}/{}", episode + 1, EPISODES);
            }
        }

        println!("--- Training Finished ---");
        for brain in &self.brains {
            brain.lock().unwrap().save_q_table()?;
        }
        Ok(())
    }

    async fn run_episode(&mut self, episode: u32) -> Result<(), SimulationError> {
        self.respawn_resources(episode);
        self.reset_players();
        self.find_and_set_valid_start_positions();

        for _step in 0..MAX_STEPS_PER_EPISODE {
            self.tick_count += 1;
            self.spawn_brain_tasks(episode).await?;
            self.run_systems();
        }
        Ok(())
    }

    fn is_day(&self) -> bool {
        (self.tick_count % (DAY_LENGTH + NIGHT_LENGTH)) < DAY_LENGTH
    }

    fn reset_players(&mut self) {
        let mut world = self.world.lock().expect("Failed to lock world");
        for i in 0..world.entities.len() {
            if let Some(player) = world.get_component_mut::<Player>(i) {
                player.reset();
            }
        }
    }

    async fn spawn_brain_tasks(&mut self, _episode: u32) -> Result<(), SimulationError> {
        let mut action_handles = Vec::new();
        {
            let world = self.world.lock().expect("Failed to lock world");
            for i in 0..world.entities.len() {
                if let Some(pos) = world.get_component::<Position>(i).map(|p| *p) {
                    if world.get_component::<Player>(i).is_some() {
                        let high_level_state = self.get_high_level_state(i)?;
                        let brain = Arc::clone(&self.brains[i]);
                        let world_clone = Arc::clone(&self.world);
                        let spatial_map_clone = self.map.spatial_map.clone();
                        let visible_tiles = self.get_visible_tiles(&pos, self.is_day());
                        let handle = task::spawn(async move {
                            let mut brain_lock = brain.lock().expect("Failed to lock brain");
                            let mut world_lock = world_clone.lock().expect("Failed to lock world");
                            brain_lock.tick(&mut world_lock, &spatial_map_clone, i, &high_level_state, &visible_tiles)
                        });
                        action_handles.push(handle);
                    }
                }
            }
        }

        let results = futures::future::join_all(action_handles).await;
        for result in results {
            match result {
                Ok(Ok(())) => {} // Brain tick succeeded, do nothing.
                Ok(Err(e)) => return Err(e), // Brain tick returned an error.
                Err(e) => return Err(SimulationError::Other(format!("Task failed: {}", e))),
            }
        }
        Ok(())
    }

    fn get_visible_tiles(&self, pos: &Position, is_day: bool) -> Vec<(Position, Tile)> {
        let mut visible_tiles = Vec::new();
        let radius = if is_day { 2 } else { 1 };
        for y in (pos.y as i32 - radius)..=(pos.y as i32 + radius) {
            for x in (pos.x as i32 - radius)..=(pos.x as i32 + radius) {
                if x >= 0 && y >= 0 && x < self.map.width as i32 && y < self.map.height as i32 {
                    let position = Position { x: x as u32, y: y as u32 };
                    let tile = self.map.grid[y as usize][x as usize].clone();
                    visible_tiles.push((position, tile));
                }
            }
        }
        visible_tiles
    }

    fn run_systems(&mut self) {
        let mut world = self.world.lock().expect("Failed to lock world");
        movement_system(&mut world);
        gathering_system(&mut world, &self.item_registry);
        crafting_system(&mut world, &self.recipe_manager, &self.item_registry);
        building_system(&mut world, &mut self.map);
        combat_system(&mut world, &self.event_bus);
        pickup_system(&mut world, &self.item_registry);
        death_system(&mut world, &self.event_bus);
    }

    fn respawn_resources(&mut self, current_episode: u32) {
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let tile = &mut self.map.grid[y as usize][x as usize];
                if let Some(depletion_episode) = tile.depletion_episode {
                    if current_episode >= depletion_episode + 4 {
                        // Find the original biome tile type
                        for biome in &self.map.biomes {
                            if biome.name == tile.biome {
                                tile.tile_type = biome.tile_type;
                                break;
                            }
                        }
                        tile.remaining_resources = Some(5);
                        tile.depletion_episode = None;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_game() -> Game {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        Game::new(
            &format!("{}/biomes.json", manifest_dir),
            &format!("{}/resources.json", manifest_dir),
            &format!("{}/items.json", manifest_dir),
            &format!("{}/recipes.json", manifest_dir),
        )
    }

    #[test]
    fn test_is_day() {
        let mut game = create_test_game();
        game.tick_count = 0;
        assert!(game.is_day());
        game.tick_count = DAY_LENGTH - 1;
        assert!(game.is_day());
        game.tick_count = DAY_LENGTH;
        assert!(!game.is_day());
        game.tick_count = DAY_LENGTH + NIGHT_LENGTH - 1;
        assert!(!game.is_day());
        game.tick_count = DAY_LENGTH + NIGHT_LENGTH;
        assert!(game.is_day());
    }

    #[test]
    fn test_get_visible_tiles() {
        let game = create_test_game();
        let pos = Position { x: 5, y: 5 };
        let visible_tiles_day = game.get_visible_tiles(&pos, true);
        assert_eq!(visible_tiles_day.len(), 25); // 5x5 area
        let visible_tiles_night = game.get_visible_tiles(&pos, false);
        assert_eq!(visible_tiles_night.len(), 9); // 3x3 area
    }
}
