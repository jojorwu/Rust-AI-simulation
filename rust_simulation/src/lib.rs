//! A simulation of a simple world with agents that can gather resources,
//! craft items, and build structures. The simulation is based on an
//! Entity-Component-System (ECS) architecture.

/// Contains the AI logic for the agents.
pub mod brain;
/// Defines the components that can be attached to entities.
pub mod components;
/// Contains configuration constants for the simulation.
pub mod config;
/// A simple Entity-Component-System (ECS) implementation.
pub mod ecs;
/// Defines the custom error types for the simulation.
pub mod errors;
/// An event bus for communication between systems.
pub mod events;
/// Field-of-view (FOV) calculations.
pub mod fov;
/// Item definitions and registry.
pub mod item;
/// Map generation and representation.
pub mod map;
/// Pathfinding logic.
pub mod pathfinding;
/// Player-specific components and logic.
pub mod player;
/// Recipe definitions and management.
pub mod recipes;
/// Road-related components and logic.
pub mod road;
/// A system for building roads.
pub mod road_builder;
/// A system for managing roads.
pub mod road_manager;
/// The systems that operate on entities and components.
pub mod systems;

use brain::{Brain, BrainAction, HighLevelState, InventorySummary};
use components::{BrainComponent, Inventory, Position};
use ecs::{Entity, World};
use errors::SimulationError;
use events::EventBus;
use item::ItemRegistry;
use map::{Map, Tile};
use player::Player;
use recipes::RecipeManager;
use std::env;
use std::sync::{Arc, Mutex};
use systems::{
    brain_event_handler_system, building_system, combat_system, crafting_system, death_system,
    gathering_system, movement_system, pickup_system, storage_system, visibility_system,
};

use rand::Rng;

use config::*;
use road_manager::RoadManager;

/// Represents the main simulation environment.
///
/// This struct holds all the state for the simulation, including the game map,
/// the Entity-Component-System (ECS) world, and the various managers for items,
/// recipes, and events.
pub struct Game {
    /// The game world, including the grid, biomes, and resources.
    pub map: Map,
    /// The ECS world, which manages all entities and their components.
    pub world: Arc<Mutex<World>>,
    /// The AI brain, which controls the behavior of the agents.
    pub brain: Arc<Brain>,
    /// The registry for all items in the simulation.
    pub item_registry: ItemRegistry,
    /// The manager for all crafting recipes.
    pub recipe_manager: Arc<RecipeManager>,
    /// The event bus for communication between systems.
    pub event_bus: Arc<Mutex<EventBus>>,
    /// The current tick count of the simulation.
    pub tick_count: u32,
    /// The manager for roads.
    pub road_manager: RoadManager,
}

impl Game {
    /// Creates a new `Game` instance.
    ///
    /// This function initializes the game state, including the map, ECS world,
    /// and all the necessary managers. It also creates the initial set of
    /// player entities.
    ///
    /// # Arguments
    ///
    /// * `biomes_path` - The path to the biomes configuration file.
    /// * `resources_path` - The path to the resources configuration file.
    /// * `items_path` - The path to the items configuration file.
    /// * `recipes_path` - The path to the recipes configuration file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Game` instance, or a `SimulationError`
    /// if initialization fails.
    pub fn new(
        biomes_path: &str,
        resources_path: &str,
        items_path: &str,
        recipes_path: &str,
    ) -> Result<Self, SimulationError> {
        let map = Map::new(WIDTH, HEIGHT, biomes_path, resources_path)?;
        let item_registry = ItemRegistry::new(items_path)?;
        let recipe_manager = Arc::new(RecipeManager::new(recipes_path)?);
        let event_bus = Arc::new(Mutex::new(EventBus::new()));

        let mut world = World::new();
        let brain = Arc::new(Brain::new(
            Arc::clone(&recipe_manager),
            LEARNING_RATE,
            DISCOUNT_FACTOR,
            EPSILON,
        ));

        for i in 0..NUM_PLAYERS {
            let player = world.create_entity();
            world.add_component(player, Player::new(i, map.width, map.height))?;
            world.add_component(player, Position { x: 0, y: 0 })?;
            world.add_component(
                player,
                crate::components::Health {
                    current: 100,
                    max: 100,
                },
            )?;
            world.add_component(player, Inventory::new())?;
            world.add_component(player, BrainComponent::new())?;
        }

        let mut game = Game {
            map,
            world: Arc::new(Mutex::new(world)),
            brain,
            item_registry,
            recipe_manager,
            event_bus,
            tick_count: 0,
            road_manager: RoadManager::new(),
        };

        game.setup_new_map()?;
        game.find_and_set_valid_start_positions()?;

        // Initial population of the spatial map
        {
            let world = game
                .world
                .lock()
                .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
            game.map.spatial_map.clear();
            for &entity in &world.entities {
                if let Some(pos) = world.get_component::<Position>(entity) {
                    game.map
                        .spatial_map
                        .entry((pos.x, pos.y))
                        .or_default()
                        .push(entity);
                }
            }
        }

        Ok(game)
    }

    fn find_and_set_valid_start_positions(&mut self) -> Result<(), SimulationError> {
        let mut rng = rand::rng();
        let mut occupied_positions = std::collections::HashSet::new();

        let plains_biome = self.map.biomes.iter().find(|b| b.name == "plains");
        let plains_tile_type = plains_biome.map_or('.', |b| b.tile_type);

        let mut world = self
            .world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
        for entity in 0..world.entities.len() {
            loop {
                let x = rng.random_range(0..self.map.width);
                let y = rng.random_range(0..self.map.height);
                if self.map.grid[y as usize][x as usize].tile_type == plains_tile_type
                    && !occupied_positions.contains(&(x, y))
                {
                    if let Some(pos) = world.get_component_mut::<Position>(entity) {
                        pos.x = x;
                        pos.y = y;
                    }
                    occupied_positions.insert((x, y));
                    break;
                }
            }
        }
        Ok(())
    }

    fn setup_new_map(&mut self) -> Result<(), SimulationError> {
        self.map.generate_island_map(25.0, 5, 0.5, 2.0);
        self.place_resources()?;
        Ok(())
    }

    fn place_resources(&mut self) -> Result<(), SimulationError> {
        let mut rng = rand::rng();
        let mut world = self
            .world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
        self.map.spatial_map.clear();

        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let tile = &self.map.grid[y as usize][x as usize];
                for resource_def in &self.map.resources {
                    if resource_def.biomes.contains(&tile.biome)
                        && rng.random::<f64>() < resource_def.density {
                            let resource_entity = world.create_entity();
                            world.add_component(resource_entity, Position { x, y })?;
                            world.add_component(
                                resource_entity,
                                crate::components::Resource {
                                    name: resource_def.name.clone(),
                                    quantity: 5, // Placeholder quantity
                                },
                            )?;
                            self.map
                                .spatial_map
                                .entry((x, y))
                                .or_default()
                                .push(resource_entity);
                            break;
                        }
                }
            }
        }
        Ok(())
    }

    fn get_high_level_state(
        &self,
        world: &World,
        entity: Entity,
    ) -> Result<HighLevelState, SimulationError> {
        let health = world
            .get_component::<crate::components::Health>(entity)
            .ok_or_else(|| SimulationError::ComponentNotFound("Health".to_string()))?;
        let inventory = world.get_component::<Inventory>(entity);
        let brain_component = world
            .get_component::<BrainComponent>(entity)
            .ok_or_else(|| SimulationError::ComponentNotFound("BrainComponent".to_string()))?;

        let num_hostile_players = brain_component
            .player_memories
            .values()
            .filter(|m| m.relationship == brain::RelationshipStatus::Hostile)
            .count() as u32;

        let inventory_summary = InventorySummary {
            has_wood: inventory.is_some_and(|inv| inv.has_item("wood", 1)),
            has_stone: inventory.is_some_and(|inv| inv.has_item("stone", 1)),
            has_iron_ore: inventory.is_some_and(|inv| inv.has_item("iron_ore", 1)),
            has_stone_axe: inventory.is_some_and(|inv| inv.has_item("stone_axe", 1)),
        };

        Ok(HighLevelState {
            inventory_summary,
            num_hostile_players,
            health_level: health.current as u32,
            is_night: !self.is_day(),
        })
    }

    /// Starts the main simulation loop.
    ///
    /// This function runs the simulation for a configured number of episodes.
    /// In each episode, it runs a series of ticks, where each tick updates
    /// the game state by running the AI brains and the various systems.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the simulation completed successfully,
    /// or a `SimulationError` if an error occurred.
    pub fn run(&mut self) -> Result<(), SimulationError> {
        println!("--- Starting Rust Training Simulation ---");

        // Run visibility system once for the initial view
        {
            let mut world = self
                .world
                .lock()
                .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
            visibility_system(&mut world, &self.map, self.is_day());
        }

        for episode in 0..EPISODES {
            self.run_episode(episode)?;
            if (episode + 1) % 200 == 0 {
                println!("Episode {}/{}", episode + 1, EPISODES);
            }
        }

        println!("--- Training Finished ---");
        // self.brain.save_q_table()?; // TODO: Figure out how to save Q-tables for multiple entities
        Ok(())
    }

    fn run_episode(&mut self, episode: u32) -> Result<(), SimulationError> {
        self.respawn_resources(episode);
        self.reset_players()?;
        self.find_and_set_valid_start_positions()?;

        for step in 0..MAX_STEPS_PER_EPISODE {
            self.tick_count += 1;
            self.run_brain_ticks(episode)?;
            self.run_systems()?;

            // Display logic moved inside the loop
            print!("\x1B[2J\x1B[1;1H"); // Clear screen
            println!(
                "--- Episode: {}/{} | Step: {}/{} ---",
                episode + 1,
                EPISODES,
                step + 1,
                MAX_STEPS_PER_EPISODE
            );
            let time_of_day = if self.is_day() { "Day" } else { "Night" };
            println!("Time: {time_of_day}");

            let world = self
                .world
                .lock()
                .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
            self.map.display(&world);
            self.map.display_observer_map(&world);

            // DEBUG: Print agent 0's status
            if let Some(brain_component) = world.get_component::<BrainComponent>(0) {
                if let Some(inventory) = world.get_component::<Inventory>(0) {
                    println!("Agent 0 Goal: {:?}", brain_component.current_goal);
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

    fn reset_players(&mut self) -> Result<(), SimulationError> {
        let mut world = self
            .world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
        for i in 0..world.entities.len() {
            if let Some(player) = world.get_component_mut::<Player>(i) {
                player.reset();
            }
        }
        Ok(())
    }

    fn run_brain_ticks(&mut self, _episode: u32) -> Result<(), SimulationError> {
        let world = self
            .world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;

        let mut brain_updates = Vec::new();

        // Phase 1: Read-only processing and collecting desired changes.
        for entity in 0..world.entities.len() {
            if let Some(brain_component) = world.get_component::<BrainComponent>(entity) {
                let pos = match world.get_component::<Position>(entity) {
                    Some(p) => *p,
                    None => continue,
                };
                let high_level_state = self.get_high_level_state(&world, entity)?;
                let visible_tiles = self.get_visible_tiles(&pos, self.is_day());

                if let Some((new_brain_state, action)) = self.brain.tick(
                    brain_component,
                    &world,
                    &self.map.spatial_map,
                    entity,
                    &high_level_state,
                    &visible_tiles,
                )? {
                    brain_updates.push((entity, new_brain_state, action));
                }
            }
        }

        // Phase 2: Apply all the collected changes with a new mutable lock.
        let mut world = self
            .world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;

        for (entity, new_brain_state, action) in brain_updates {
            // Update the brain component
            if let Some(brain_component) = world.get_component_mut::<BrainComponent>(entity) {
                *brain_component = new_brain_state;
            }

            // Apply the action
            match action {
                BrainAction::Move(vel) => world.add_component(entity, vel)?,
                BrainAction::Gather(g) => world.add_component(entity, g)?,
                BrainAction::Craft(c) => world.add_component(entity, c)?,
                BrainAction::Build(b) => world.add_component(entity, b)?,
                BrainAction::Attack(a) => world.add_component(entity, a)?,
                BrainAction::Store(s) => world.add_component(entity, s)?,
            }
        }

        Ok(())
    }

    fn get_visible_tiles(&self, pos: &Position, is_day: bool) -> Vec<(Position, Tile)> {
        let radius = if is_day { 8 } else { 4 }; // Use a larger radius for day
        let visible_positions = fov::field_of_view(pos, radius, &self.map);

        visible_positions
            .iter()
            .map(|position| {
                let tile = self.map.grid[position.y as usize][position.x as usize].clone();
                (*position, tile)
            })
            .collect()
    }

    fn run_systems(&mut self) -> Result<(), SimulationError> {
        let mut world = self
            .world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
        visibility_system(&mut world, &self.map, self.is_day());
        movement_system(&mut world, &mut self.map);
        gathering_system(&mut world, &self.item_registry);
        crafting_system(&mut world, &self.recipe_manager, &self.item_registry);
        building_system(
            &mut world,
            &mut self.map,
            &self.event_bus,
            &self.recipe_manager,
        )?;
        storage_system(&mut world);
        combat_system(&mut world, &self.event_bus)?;
        pickup_system(&mut world, &self.item_registry, &mut self.map);
        death_system(&mut world, &self.event_bus, &mut self.map)?;
        brain_event_handler_system(&mut world, &self.event_bus)?;
        Ok(())
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

    /// Wipes the current simulation state and starts a new generation.
    ///
    /// This function is used to reset the simulation to a clean state. It
    /// deletes the old Q-table, resets the game state, creates a new ECS
    /// world, and generates a new map with new resources.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the new generation was created
    /// successfully, or a `SimulationError` if an error occurred.
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

        // 2. Create a new World and re-initialize brain
        let mut world = World::new();
        self.brain = Arc::new(Brain::new(
            Arc::clone(&self.recipe_manager),
            LEARNING_RATE,
            DISCOUNT_FACTOR,
            EPSILON,
        ));

        for i in 0..NUM_PLAYERS {
            let player = world.create_entity();
            world.add_component(
                player,
                Player::new(i, self.map.width, self.map.height),
            )?;
            world.add_component(player, Position { x: 0, y: 0 })?;
            world.add_component(
                player,
                crate::components::Health {
                    current: 100,
                    max: 100,
                },
            )?;
            world.add_component(player, Inventory::new())?;
            world.add_component(player, BrainComponent::new())?;
        }

        self.world = Arc::new(Mutex::new(world));

        // 3. Generate a new map and place resources
        self.setup_new_map()?;

        // 4. Set starting positions for players
        self.find_and_set_valid_start_positions()?;

        println!("--- New generation is ready ---");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::SimulationError;
    use crate::map::TileState;
    use std::env;

    fn create_test_game() -> Result<Game, SimulationError> {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| SimulationError::EnvVarError(e.to_string()))?;
        Game::new(
            &format!("{manifest_dir}/data/biomes.json"),
            &format!("{manifest_dir}/data/resources.json"),
            &format!("{manifest_dir}/data/items.json"),
            &format!("{manifest_dir}/data/recipes.json"),
        )
    }

    #[test]
    fn test_is_day() -> Result<(), SimulationError> {
        let mut game = create_test_game()?;
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
        Ok(())
    }

    #[test]
    fn test_get_visible_tiles() -> Result<(), SimulationError> {
        // Create a game with a blank map (no walls) to make the test deterministic.
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| SimulationError::EnvVarError(e.to_string()))?;
        let mut game = Game::new(
            &format!("{manifest_dir}/data/biomes.json"),
            &format!("{manifest_dir}/data/resources.json"),
            &format!("{manifest_dir}/data/items.json"),
            &format!("{manifest_dir}/data/recipes.json"),
        )?;
        game.map.grid = vec![vec![Tile::new('.', "plains".to_string()); 100]; 100];

        let pos = Position { x: 50, y: 50 }; // Use center of map to avoid edge effects

        let visible_tiles_day = game.get_visible_tiles(&pos, true); // radius = 8
        assert_eq!(visible_tiles_day.len(), 197);

        let visible_tiles_night = game.get_visible_tiles(&pos, false); // radius = 4
        assert_eq!(visible_tiles_night.len(), 49);
        Ok(())
    }

    #[test]
    fn test_visibility_system_day_night() -> Result<(), SimulationError> {
        let mut game = create_test_game()?;
        // Create a blank map for deterministic results
        game.map.grid = vec![vec![Tile::new('.', "plains".to_string()); 100]; 100];
        let player_entity = 0;

        {
            let mut world = game
                .world
                .lock()
                .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
            // Set player position to the center for predictable FOV
            if let Some(pos) = world.get_component_mut::<Position>(player_entity) {
                pos.x = 50;
                pos.y = 50;
            }
        }

        // --- Test Day ---
        game.tick_count = 0; // Day time
        {
            let mut world = game
                .world
                .lock()
                .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
            visibility_system(&mut world, &game.map, game.is_day());

            let player = world
                .get_component::<Player>(player_entity)
                .ok_or_else(|| SimulationError::ComponentNotFound("Player".to_string()))?;
            let visible_count = player
                .mental_map
                .grid
                .iter()
                .flatten()
                .filter(|&&s| s == TileState::Visible)
                .count();
            assert_eq!(visible_count, 197, "Visible tiles on a clear day");
        }

        // --- Test Night ---
        game.tick_count = DAY_LENGTH; // Night time
        {
            let mut world = game
                .world
                .lock()
                .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
            visibility_system(&mut world, &game.map, game.is_day());

            let player = world
                .get_component::<Player>(player_entity)
                .ok_or_else(|| SimulationError::ComponentNotFound("Player".to_string()))?;
            let visible_count = player
                .mental_map
                .grid
                .iter()
                .flatten()
                .filter(|&&s| s == TileState::Visible)
                .count();
            assert_eq!(visible_count, 49, "Visible tiles on a clear night");
        }
        Ok(())
    }

    #[test]
    fn test_trees_block_vision() -> Result<(), SimulationError> {
        let mut game = create_test_game()?;
        let player_entity = 0;
        let player_pos = Position { x: 50, y: 50 };
        let tree_pos = Position { x: 51, y: 51 };
        let target_pos = Position { x: 52, y: 52 };

        // Set player position
        {
            let mut world = game
                .world
                .lock()
                .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
            if let Some(pos) = world.get_component_mut::<Position>(player_entity) {
                *pos = player_pos;
            }
        }

        // --- Test with tree blocking vision ---
        game.map.grid = vec![vec![Tile::new('.', "plains".to_string()); 100]; 100];
        game.map.grid[tree_pos.y as usize][tree_pos.x as usize].tile_type = 'T'; // Place a tree

        let visible_tiles = game.get_visible_tiles(&player_pos, true);
        let visible_positions: std::collections::HashSet<Position> =
            visible_tiles.into_iter().map(|(p, _)| p).collect();

        assert!(
            visible_positions.contains(&tree_pos),
            "Tree should be visible"
        );
        assert!(
            !visible_positions.contains(&target_pos),
            "Tile behind tree should not be visible"
        );

        // --- Test without tree ---
        game.map.grid[tree_pos.y as usize][tree_pos.x as usize].tile_type = '.'; // Remove the tree

        let visible_tiles_no_tree = game.get_visible_tiles(&player_pos, true);
        let visible_positions_no_tree: std::collections::HashSet<Position> =
            visible_tiles_no_tree.into_iter().map(|(p, _)| p).collect();

        assert!(
            visible_positions_no_tree.contains(&target_pos),
            "Tile should be visible without tree"
        );
        Ok(())
    }
}
