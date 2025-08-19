use super::map::{Map, Tile};
use super::player::Player;
use super::brain::{Brain, HighLevelState};
use super::recipes::RecipeManager;
use super::errors::SimulationError;
use super::item::ItemRegistry;
use super::ecs::{World, Entity};
use super::components::{Position, Inventory};
use super::events::EventBus;
use super::systems::{visibility_system, movement_system, gathering_system, crafting_system, building_system, combat_system, pickup_system, death_system};
use std::sync::{Arc, Mutex};
use crate::fov;
use std::env;
use std::collections::HashMap;

use rand::Rng;

use super::config::*;
use super::road::*;
use super::road_manager::RoadManager;


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
    pub road_manager: RoadManager,
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
            world.add_component(player, Player::new(i as u32, map.width, map.height));
            world.add_component(player, Position { x: 0, y: 0 });
            world.add_component(player, crate::components::Health { current: 100, max: 100 });
            world.add_component(player, Inventory::new());
            brains.push(Arc::new(Mutex::new(Brain::new(Arc::clone(&recipe_manager), 0.1, 0.9, 1.0))));
        }

        let mut game = Game {
            map,
            world: Arc::new(Mutex::new(world)),
            brains,
            item_registry,
            recipe_manager,
            event_bus,
            tick_count: 0,
            road_manager: RoadManager::new(),
        };

        game.setup_new_map();
        game.generate_roads_from_config().expect("Failed to generate roads");
        game.find_and_set_valid_start_positions();

        // Initial population of the spatial map
        {
            let world = game.world.lock().unwrap();
            game.map.spatial_map.clear();
            for &entity in &world.entities {
                if let Some(pos) = world.get_component::<Position>(entity) {
                    game.map.spatial_map.entry((pos.x, pos.y)).or_default().push(entity);
                }
            }
        }

        game
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

    fn generate_roads_from_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let road_config = RoadConfig::load("road_config.json")?;

        // This is a placeholder. In a real game, these locations might be dynamically determined.
        let mut city_locations: HashMap<String, (i32, i32)> = HashMap::new();
        city_locations.insert("CityA".to_string(), (20, 20));
        city_locations.insert("CityB".to_string(), (80, 30));
        city_locations.insert("CityC".to_string(), (30, 70));
        city_locations.insert("CityD".to_string(), (90, 85));
        city_locations.insert("Old_Mine".to_string(), (50, 90));


        for setting in road_config.road_settings {
            let start_pos = city_locations.get(&setting.start_point).ok_or("Start city not found")?;
            let end_pos = city_locations.get(&setting.end_point).ok_or("End city not found")?;

            let start_point = Point { x: start_pos.0 as f32, y: start_pos.1 as f32 };
            let end_point = Point { x: end_pos.0 as f32, y: end_pos.1 as f32 };

            let road = super::road::generate_road(setting, start_point, end_point);

            for point in &road.path {
                if point.x >= 0.0 && point.x < self.map.width as f32 && point.y >= 0.0 && point.y < self.map.height as f32 {
                    let x = point.x as usize;
                    let y = point.y as usize;
                    let tile = &mut self.map.grid[y][x];
                    tile.tile_type = '=';
                }
            }
            self.road_manager.add_road(road);
        }
        Ok(())
    }

    fn get_high_level_state(&self, entity: Entity) -> Result<HighLevelState, SimulationError> {
        let world = self.world.lock().expect("Failed to lock world");
        let health = world.get_component::<crate::components::Health>(entity)
            .ok_or_else(|| SimulationError::ComponentNotFound("Health".to_string()))?;
        let inventory = world.get_component::<Inventory>(entity);

        let brain_lock = self.brains[entity].lock().expect("Failed to lock brain");
        let num_hostile_players = brain_lock.player_memories.values().filter(|m| m.relationship == super::brain::RelationshipStatus::Hostile).count() as u32;

        Ok(HighLevelState {
            has_wood: inventory.map_or(false, |inv| inv.has_item("wood", 1)),
            has_stone: inventory.map_or(false, |inv| inv.has_item("stone", 1)),
            has_iron_ore: inventory.map_or(false, |inv| inv.has_item("iron_ore", 1)),
            has_stone_axe: inventory.map_or(false, |inv| inv.has_item("stone_axe", 1)),
            num_hostile_players,
            health_level: health.current as u32,
            is_night: !self.is_day(),
        })
    }

    pub fn run(&mut self) -> Result<(), SimulationError> {
        println!("--- Starting Rust Training Simulation ---");

        // Run visibility system once for the initial view
        {
            let mut world = self.world.lock().expect("Failed to lock world");
            visibility_system(&mut world, &self.map, self.is_day());
        }

        for episode in 0..EPISODES {
            self.run_episode(episode)?;
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

    fn run_episode(&mut self, episode: u32) -> Result<(), SimulationError> {
        self.respawn_resources(episode);
        self.reset_players();
        self.find_and_set_valid_start_positions();

        for step in 0..MAX_STEPS_PER_EPISODE {
            self.tick_count += 1;
            self.run_brain_ticks(episode)?;
            self.run_systems();

            // Display logic moved inside the loop
            print!("\x1B[2J\x1B[1;1H"); // Clear screen
            println!("--- Episode: {}/{} | Step: {}/{} ---", episode + 1, EPISODES, step + 1, MAX_STEPS_PER_EPISODE);
            let time_of_day = if self.is_day() { "Day" } else { "Night" };
            println!("Time: {}", time_of_day);

            self.map.display(&self.world.lock().expect("Failed to lock world"));
            self.map.display_observer_map(&self.world.lock().expect("Failed to lock world"));

            // DEBUG: Print agent 0's status
            if self.brains.len() > 0 {
                let brain = self.brains[0].lock().unwrap();
                let world = self.world.lock().unwrap();
                if let Some(inventory) = world.get_component::<Inventory>(0) {
                    println!("Agent 0 Goal: {:?}", brain.current_goal);
                    println!("Agent 0 Inventory: {:?}", inventory.items);
                }
            }

            // Add a small delay to make the animation viewable
            std::thread::sleep(std::time::Duration::from_millis(100));
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

    fn run_brain_ticks(&mut self, _episode: u32) -> Result<(), SimulationError> {
        for i in 0..self.brains.len() {
            let pos = {
                let world = self.world.lock().unwrap();
                if let Some(p) = world.get_component::<Position>(i) {
                    *p
                } else {
                    continue;
                }
            };

            let high_level_state = self.get_high_level_state(i)?;
            let visible_tiles = self.get_visible_tiles(&pos, self.is_day());

            let brain = Arc::clone(&self.brains[i]);
            let mut brain_lock = brain.lock().expect("Failed to lock brain");

            // The world is locked for the duration of the tick
            let mut world_lock = self.world.lock().expect("Failed to lock world");
            brain_lock.tick(&mut world_lock, &self.map.spatial_map, i, &high_level_state, &visible_tiles)?;
        }
        Ok(())
    }

    fn get_visible_tiles(&self, pos: &Position, is_day: bool) -> Vec<(Position, Tile)> {
        let radius = if is_day { 8 } else { 4 }; // Use a larger radius for day
        let visible_positions = fov::field_of_view(pos, radius, &self.map);

        visible_positions.iter().map(|position| {
            let tile = self.map.grid[position.y as usize][position.x as usize].clone();
            (*position, tile)
        }).collect()
    }

    fn run_systems(&mut self) {
        let mut world = self.world.lock().expect("Failed to lock world");
        visibility_system(&mut world, &self.map, self.is_day());
        movement_system(&mut world, &mut self.map);
        gathering_system(&mut world, &self.item_registry);
        crafting_system(&mut world, &self.recipe_manager, &self.item_registry);
        building_system(&mut world, &mut self.map);
        combat_system(&mut world, &self.event_bus);
        pickup_system(&mut world, &self.item_registry, &mut self.map);
        death_system(&mut world, &self.event_bus, &mut self.map);
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

    pub fn new_generation(&mut self) -> Result<(), SimulationError> {
        println!("--- Wiping and creating a new generation ---");

        // Delete the old Q-table to ensure a fresh start
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let q_table_path = std::path::Path::new(manifest_dir).join("../q_table.json");
        if std::fs::remove_file(q_table_path).is_ok() {
            println!("Removed old q_table.json");
        }

        // 1. Reset core game state
        self.tick_count = 0;
        self.event_bus = Arc::new(Mutex::new(EventBus::new()));

        // 2. Create a new World and re-initialize brains
        let mut world = World::new();
        let mut brains = Vec::new();

        for i in 0..NUM_PLAYERS {
            let player = world.create_entity();
            world.add_component(player, Player::new(i as u32, self.map.width, self.map.height));
            world.add_component(player, Position { x: 0, y: 0 });
            world.add_component(player, crate::components::Health { current: 100, max: 100 });
            world.add_component(player, Inventory::new());
            // This will create a new brain, which will either load a q-table or start fresh
            brains.push(Arc::new(Mutex::new(Brain::new(Arc::clone(&self.recipe_manager), 0.1, 0.9, 1.0))));
        }

        self.world = Arc::new(Mutex::new(world));
        self.brains = brains;

        // 3. Generate a new map and place resources
        self.setup_new_map();

        // 4. Set starting positions for players
        self.find_and_set_valid_start_positions();

        println!("--- New generation is ready ---");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::TileState;
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
        // Create a game with a blank map (no walls) to make the test deterministic.
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut game = Game::new(
            &format!("{}/biomes.json", manifest_dir),
            &format!("{}/resources.json", manifest_dir),
            &format!("{}/items.json", manifest_dir),
            &format!("{}/recipes.json", manifest_dir),
        );
        game.map.grid = vec![vec![Tile::new('.', "plains".to_string()); 100]; 100];


        let pos = Position { x: 50, y: 50 }; // Use center of map to avoid edge effects

        let visible_tiles_day = game.get_visible_tiles(&pos, true); // radius = 8
        assert_eq!(visible_tiles_day.len(), 197);

        let visible_tiles_night = game.get_visible_tiles(&pos, false); // radius = 4
        assert_eq!(visible_tiles_night.len(), 49);
    }

    #[test]
    fn test_visibility_system_day_night() {
        let mut game = create_test_game();
        // Create a blank map for deterministic results
        game.map.grid = vec![vec![Tile::new('.', "plains".to_string()); 100]; 100];
        let player_entity = 0;

        {
            let mut world = game.world.lock().unwrap();
            // Set player position to the center for predictable FOV
            if let Some(pos) = world.get_component_mut::<Position>(player_entity) {
                pos.x = 50;
                pos.y = 50;
            }
        }

        // --- Test Day ---
        game.tick_count = 0; // Day time
        {
            let mut world = game.world.lock().unwrap();
            visibility_system(&mut world, &game.map, game.is_day());

            let player = world.get_component::<Player>(player_entity).unwrap();
            let visible_count = player.mental_map.grid.iter().flatten().filter(|&&s| s == TileState::Visible).count();
            assert_eq!(visible_count, 197, "Visible tiles on a clear day");
        }

        // --- Test Night ---
        game.tick_count = DAY_LENGTH; // Night time
        {
            let mut world = game.world.lock().unwrap();
            visibility_system(&mut world, &game.map, game.is_day());

            let player = world.get_component::<Player>(player_entity).unwrap();
            let visible_count = player.mental_map.grid.iter().flatten().filter(|&&s| s == TileState::Visible).count();
            assert_eq!(visible_count, 49, "Visible tiles on a clear night");
        }
    }

    #[test]
    fn test_trees_block_vision() {
        let mut game = create_test_game();
        let player_entity = 0;
        let player_pos = Position { x: 50, y: 50 };
        let tree_pos = Position { x: 51, y: 51 };
        let target_pos = Position { x: 52, y: 52 };

        // Set player position
        {
            let mut world = game.world.lock().unwrap();
            if let Some(pos) = world.get_component_mut::<Position>(player_entity) {
                *pos = player_pos;
            }
        }

        // --- Test with tree blocking vision ---
        game.map.grid = vec![vec![Tile::new('.', "plains".to_string()); 100]; 100];
        game.map.grid[tree_pos.y as usize][tree_pos.x as usize].tile_type = 'T'; // Place a tree

        let visible_tiles = game.get_visible_tiles(&player_pos, true);
        let visible_positions: std::collections::HashSet<Position> = visible_tiles.into_iter().map(|(p, _)| p).collect();

        assert!(visible_positions.contains(&tree_pos), "Tree should be visible");
        assert!(!visible_positions.contains(&target_pos), "Tile behind tree should not be visible");

        // --- Test without tree ---
        game.map.grid[tree_pos.y as usize][tree_pos.x as usize].tile_type = '.'; // Remove the tree

        let visible_tiles_no_tree = game.get_visible_tiles(&player_pos, true);
        let visible_positions_no_tree: std::collections::HashSet<Position> = visible_tiles_no_tree.into_iter().map(|(p, _)| p).collect();

        assert!(visible_positions_no_tree.contains(&target_pos), "Tile should be visible without tree");
    }
}
