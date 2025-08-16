use super::map::Map;
use super::player::Player;
use super::brain::{Brain, HighLevelState};
use super::recipes::RecipeManager;
use super::errors::SimulationError;
use super::actions::{get_all_actions};
use super::item::ItemRegistry;
use super::ecs::{World, Entity};
use super::components::Position;
use super::systems::{movement_system, gathering_system, crafting_system, building_system, combat_system, pickup_system};
use std::sync::{Arc, Mutex};
use tokio::task;

use rand::Rng;

use super::config::*;


pub struct Game {
    pub map: Map,
    pub world: Arc<Mutex<World>>,
    pub brains: Vec<Arc<Mutex<Brain>>>,
    pub item_registry: ItemRegistry,
    pub recipe_manager: Arc<RecipeManager>,
    _next_instance_id: u32,
}


impl Game {
    pub fn new() -> Self {
        let map = Map::new(WIDTH, HEIGHT).expect("Failed to create map");
        let item_registry = ItemRegistry::new("items.json");
        let recipe_manager = Arc::new(RecipeManager::new("recipes.json"));

        let mut world = World::new();
        let mut brains = Vec::new();
        let actions = get_all_actions();

        for i in 0..NUM_PLAYERS {
            let player = world.create_entity();
            world.add_component(player, Player::new(i as u32));
            world.add_component(player, Position { x: 0, y: 0 });
            world.add_component(player, crate::components::Health { current: 100, max: 100 });
            brains.push(Arc::new(Mutex::new(Brain::new(actions.clone(), Arc::clone(&recipe_manager), 0.1, 0.9, 1.0))));
        }

        Game {
            map,
            world: Arc::new(Mutex::new(world)),
            brains,
            item_registry,
            recipe_manager,
            _next_instance_id: 0,
        }
    }

    fn _find_and_set_valid_start_positions(&mut self) {
        let mut rng = rand::thread_rng();
        let mut occupied_positions = std::collections::HashSet::new();

        let plains_biome = self.map.biomes.iter().find(|b| b.name == "plains");
        let plains_tile_type = plains_biome.map_or('.', |b| b.tile_type);

        let mut world = self.world.lock().unwrap();
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
        self._place_resources();
    }

    fn _place_resources(&mut self) {
        let mut rng = rand::thread_rng();
        let mut world = self.world.lock().unwrap();

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
                            break;
                        }
                    }
                }
            }
        }
    }

    fn get_high_level_state(&self, entity: Entity) -> HighLevelState {
        let world = self.world.lock().unwrap();
        let player = world.get_component::<Player>(entity).unwrap();
        let health = world.get_component::<crate::components::Health>(entity).unwrap();
        let brain_lock = self.brains[entity].lock().unwrap();

        let num_hostile_players = brain_lock.player_memories.values().filter(|m| m.relationship == super::brain::RelationshipStatus::Hostile).count() as u32;

        HighLevelState {
            has_wood: player.get_total_quantity("wood") > 0,
            has_stone: player.get_total_quantity("stone") > 0,
            has_iron_ore: player.get_total_quantity("iron_ore") > 0,
            has_stone_axe: player.get_total_quantity("stone_axe") > 0,
            num_hostile_players,
            health_level: health.current as u32,
        }
    }

    pub async fn run(&mut self) -> Result<(), SimulationError> {
        println!("--- Starting Rust Training Simulation ---");
        self.setup_new_map();
        self._find_and_set_valid_start_positions();


        println!("Initial Map:");
        self.map.display(&self.world.lock().unwrap());

        for episode in 0..EPISODES {
            self._respawn_resources(episode);
            {
                let mut world = self.world.lock().unwrap();
                for i in 0..world.entities.len() {
                    if let Some(player) = world.get_component_mut::<Player>(i) {
                        player.reset();
                    }
                }
            }
            self._find_and_set_valid_start_positions();

            for _step in 0..MAX_STEPS_PER_EPISODE {
                let mut action_handles = Vec::new();

                {
                    let world = self.world.lock().unwrap();
                    for i in 0..world.entities.len() {
                        if world.get_component::<Player>(i).is_some() {
                            let high_level_state = self.get_high_level_state(i);
                            let brain = Arc::clone(&self.brains[i]);
                            let world_clone = Arc::clone(&self.world);
                            let handle = task::spawn(async move {
                                let mut brain_lock = brain.lock().unwrap();
                                let mut world_lock = world_clone.lock().unwrap();
                                brain_lock.tick(&mut world_lock, i, &high_level_state, episode)
                            });
                            action_handles.push(handle);
                        }
                    }
                }

                let results = futures::future::join_all(action_handles).await;
                for result in results {
                    match result {
                        Ok(Ok(())) => {
                            // Brain tick succeeded, do nothing.
                        },
                        Ok(Err(e)) => {
                            // Brain tick returned an error.
                            return Err(e);
                        },
                        Err(e) => {
                            // The tokio task failed to execute (e.g., panicked).
                            return Err(SimulationError::Other(format!("Task failed: {}", e)));
                        }
                    }
                }

                movement_system(&mut self.world.lock().unwrap());
                gathering_system(&mut self.world.lock().unwrap(), &self.item_registry);
                crafting_system(&mut self.world.lock().unwrap(), &self.recipe_manager, &self.item_registry);
                building_system(&mut self.world.lock().unwrap(), &mut self.map);
                combat_system(&mut self.world.lock().unwrap());
                pickup_system(&mut self.world.lock().unwrap(), &self.item_registry);
            }

            if (episode + 1) % 200 == 0 {
                println!("Episode {}/{}", episode + 1, EPISODES);
            }
        }

        println!("--- Training Finished ---");
        Ok(())
    }

    fn _respawn_resources(&mut self, current_episode: u32) {
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

