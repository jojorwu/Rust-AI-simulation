//! A simulation of a simple world with agents that can gather resources,
//! craft items, and build structures. The simulation is based on an
//! Entity-Component-System (ECS) architecture.

pub mod brain;
pub mod components;
pub mod config;
pub mod ecs;
pub mod errors;
pub mod events;
pub mod fov;
pub mod item;
pub mod map;
pub mod pathfinding;
pub mod player;
pub mod recipes;
pub mod road;
pub mod road_builder;
pub mod road_manager;
pub mod systems;
pub mod renderer;
pub mod world;

use brain::{Brain, BrainAction, HighLevelState, InventorySummary};
use components::{BrainComponent, Inventory, Position};
use ecs::{Entity, World as EcsWorld};
use errors::SimulationError;
use events::EventBus;
use item::ItemRegistry;
use map::{Map, Tile};
use player::Player;
use recipes::RecipeManager;
use std::env;
use std::sync::{Arc, Mutex};
use systems::Scheduler;
use world::ParallelGameState;

use rand::Rng;

use config::*;
use road_manager::RoadManager;

/// Represents the main simulation environment.
pub struct Game {
    pub parallel_state: ParallelGameState,
    pub brain: Arc<Brain>,
    pub tick_count: u32,
    pub road_manager: RoadManager,
    scheduler: Scheduler,
}

impl Game {
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

        let mut ecs_world = EcsWorld::new();
        let brain = Arc::new(Brain::new(
            Arc::clone(&recipe_manager),
            LEARNING_RATE,
            DISCOUNT_FACTOR,
            EPSILON,
        ));

        for i in 0..NUM_PLAYERS {
            let player = ecs_world.create_entity();
            ecs_world.add_component(player, Player::new(i, map.width, map.height))?;
            ecs_world.add_component(player, Position { x: 0, y: 0 })?;
            ecs_world.add_component(
                player,
                crate::components::Health {
                    current: 100,
                    max: 100,
                },
            )?;
            ecs_world.add_component(player, Inventory::new())?;
            ecs_world.add_component(player, BrainComponent::new())?;
        }

        let mut game = Game {
            parallel_state: ParallelGameState {
                map,
                world: Arc::new(Mutex::new(ecs_world)),
                item_registry,
                recipe_manager,
                event_bus,
            },
            brain,
            tick_count: 0,
            road_manager: RoadManager::new(),
            scheduler: Scheduler::new(),
        };

        game.setup_new_map()?;
        game.find_and_set_valid_start_positions()?;

        {
            let world = game
                .parallel_state.world
                .lock()
                .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
            game.parallel_state.map.spatial_map.clear();
            for &entity in &world.entities {
                if let Some(pos) = world.get_component::<Position>(entity) {
                    game.parallel_state.map
                        .spatial_map
                        .entry((pos.x, pos.y))
                        .or_default()
                        .push(entity);
                }
            }
        }

        Ok(game)
    }

    pub fn tick(&mut self) -> Result<(), SimulationError> {
        if self.tick_count % MAX_STEPS_PER_EPISODE == 0 {
            let current_episode = self.tick_count / MAX_STEPS_PER_EPISODE;
            self.start_new_episode(current_episode)?;
        }

        self.tick_count += 1;
        self.run_brain_ticks()?;
        self.run_systems()?;
        Ok(())
    }

    fn start_new_episode(&mut self, episode: u32) -> Result<(), SimulationError> {
        self.respawn_resources(episode);
        self.reset_players()?;
        self.find_and_set_valid_start_positions()?;
        Ok(())
    }

    fn find_and_set_valid_start_positions(&mut self) -> Result<(), SimulationError> {
        let mut rng = rand::rng();
        let mut occupied_positions = std::collections::HashSet::new();

        let plains_biome = self.parallel_state.map.biomes.iter().find(|b| b.name == "plains");
        let plains_tile_type = plains_biome.map_or('.', |b| b.tile_type);

        let mut world = self
            .parallel_state.world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
        for entity in 0..world.entities.len() {
            loop {
                let x = rng.random_range(0..self.parallel_state.map.width);
                let y = rng.random_range(0..self.parallel_state.map.height);
                if self.parallel_state.map.grid[y as usize][x as usize].tile_type == plains_tile_type
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
        self.parallel_state.map.generate_island_map(25.0, 5, 0.5, 2.0);
        self.place_resources()?;
        Ok(())
    }

    fn place_resources(&mut self) -> Result<(), SimulationError> {
        let mut rng = rand::rng();
        let mut world = self
            .parallel_state.world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
        self.parallel_state.map.spatial_map.clear();

        for y in 0..self.parallel_state.map.height {
            for x in 0..self.parallel_state.map.width {
                let tile = &self.parallel_state.map.grid[y as usize][x as usize];
                for resource_def in &self.parallel_state.map.resources {
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
                            self.parallel_state.map
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
        world: &EcsWorld,
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

    pub fn is_day(&self) -> bool {
        (self.tick_count % (DAY_LENGTH + NIGHT_LENGTH)) < DAY_LENGTH
    }

    fn reset_players(&mut self) -> Result<(), SimulationError> {
        let mut world = self
            .parallel_state.world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
        for i in 0..world.entities.len() {
            if let Some(player) = world.get_component_mut::<Player>(i) {
                player.reset();
            }
        }
        Ok(())
    }

    fn run_brain_ticks(&mut self) -> Result<(), SimulationError> {
        let world = self
            .parallel_state.world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;

        let mut brain_updates = Vec::new();

        for entity in 0..world.entities.len() {
            if let Some(brain_component) = world.get_component::<BrainComponent>(entity) {
                let pos = match world.get_component::<Position>(entity) {
                    Some(p) => *p,
                    None => continue,
                };
                let high_level_state = self.get_high_level_state(&world, entity)?;
                let visible_tiles = self.get_visible_tiles(&pos, self.is_day());

                if let Some((update, action)) = self.brain.tick(
                    brain_component,
                    &world,
                    &self.parallel_state.map.spatial_map,
                    entity,
                    &high_level_state,
                    &visible_tiles,
                )? {
                    brain_updates.push((entity, update, action));
                }
            }
        }

        let mut world = self
            .parallel_state.world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;

        for (entity, update, action) in brain_updates {
            if let Some(brain_component) = world.get_component_mut::<BrainComponent>(entity) {
                brain_component.current_goal = update.current_goal;
                brain_component.goal_stack = update.goal_stack;
                brain_component.current_path = update.current_path;
                brain_component.goal_commitment_ticks = update.goal_commitment_ticks;
                brain_component.prev_state = update.prev_state;
                brain_component.prev_goal = update.prev_goal;
            }

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
        let radius = if is_day { 8 } else { 4 };
        let visible_positions = fov::field_of_view(pos, radius, &self.parallel_state.map);

        visible_positions
            .iter()
            .map(|position| {
                let tile = self.parallel_state.map.grid[position.y as usize][position.x as usize].clone();
                (*position, tile)
            })
            .collect()
    }

    fn run_systems(&mut self) -> Result<(), SimulationError> {
        let is_day = self.is_day();
        self.scheduler
            .run_parallel(&mut self.parallel_state, is_day)
    }

    fn respawn_resources(&mut self, current_episode: u32) {
        for y in 0..self.parallel_state.map.height {
            for x in 0..self.parallel_state.map.width {
                let tile = &mut self.parallel_state.map.grid[y as usize][x as usize];
                if let Some(depletion_episode) = tile.depletion_episode {
                    if current_episode >= depletion_episode + 4 {
                        for biome in &self.parallel_state.map.biomes {
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
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let q_table_path = std::path::Path::new(manifest_dir).join("../q_table.json");
        if std::fs::remove_file(q_table_path).is_ok() {
        }

        self.tick_count = 0;
        self.parallel_state.event_bus = Arc::new(Mutex::new(EventBus::new()));

        let mut world = EcsWorld::new();
        self.brain = Arc::new(Brain::new(
            Arc::clone(&self.parallel_state.recipe_manager),
            LEARNING_RATE,
            DISCOUNT_FACTOR,
            EPSILON,
        ));

        for i in 0..NUM_PLAYERS {
            let player = world.create_entity();
            world.add_component(
                player,
                Player::new(i, self.parallel_state.map.width, self.parallel_state.map.height),
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

        self.parallel_state.world = Arc::new(Mutex::new(world));

        self.setup_new_map()?;
        self.find_and_set_valid_start_positions()?;

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
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| SimulationError::EnvVarError(e.to_string()))?;
        let mut game = Game::new(
            &format!("{manifest_dir}/data/biomes.json"),
            &format!("{manifest_dir}/data/resources.json"),
            &format!("{manifest_dir}/data/items.json"),
            &format!("{manifest_dir}/data/recipes.json"),
        )?;
        game.parallel_state.map.grid = vec![vec![Tile::new('.', "plains".to_string()); 100]; 100];

        let pos = Position { x: 50, y: 50 };

        let visible_tiles_day = game.get_visible_tiles(&pos, true);
        assert_eq!(visible_tiles_day.len(), 197);

        let visible_tiles_night = game.get_visible_tiles(&pos, false);
        assert_eq!(visible_tiles_night.len(), 49);
        Ok(())
    }

    // #[test]
    // fn test_visibility_system_day_night() -> Result<(), SimulationError> {
    //     let mut game = create_test_game()?;
    //     game.parallel_state.map.grid = vec![vec![Tile::new('.', "plains".to_string()); 100]; 100];
    //     let player_entity = 0;

    //     {
    //         let mut world = game
    //             .parallel_state.world
    //             .lock()
    //             .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
    //         if let Some(pos) = world.get_component_mut::<Position>(player_entity) {
    //             pos.x = 50;
    //             pos.y = 50;
    //         }
    //     }

    //     game.tick_count = 0; // Day time
    //     {
    //         let mut world = game
    //             .parallel_state.world
    //             .lock()
    //             .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
    //         visibility_system(&mut world, &game.parallel_state.map, game.is_day());

    //         let player = world
    //             .get_component::<Player>(player_entity)
    //             .ok_or_else(|| SimulationError::ComponentNotFound("Player".to_string()))?;
    //         let visible_count = player
    //             .mental_map
    //             .grid
    //             .iter()
    //             .flatten()
    //             .filter(|&&s| s == TileState::Visible)
    //             .count();
    //         assert_eq!(visible_count, 197, "Visible tiles on a clear day");
    //     }

    //     game.tick_count = DAY_LENGTH; // Night time
    //     {
    //         let mut world = game
    //             .parallel_state.world
    //             .lock()
    //             .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
    //         visibility_system(&mut world, &game.parallel_state.map, game.is_day());

    //         let player = world
    //             .get_component::<Player>(player_entity)
    //             .ok_or_else(|| SimulationError::ComponentNotFound("Player".to_string()))?;
    //         let visible_count = player
    //             .mental_map
    //             .grid
    //             .iter()
    //             .flatten()
    //             .filter(|&&s| s == TileState::Visible)
    //             .count();
    //         assert_eq!(visible_count, 49, "Visible tiles on a clear night");
    //     }
    //     Ok(())
    // }

    // #[test]
    // fn test_trees_block_vision() -> Result<(), SimulationError> {
    //     let mut game = create_test_game()?;
    //     let player_entity = 0;
    //     let player_pos = Position { x: 50, y: 50 };
    //     let tree_pos = Position { x: 51, y: 51 };
    //     let target_pos = Position { x: 52, y: 52 };

    //     {
    //         let mut world = game
    //             .parallel_state.world
    //             .lock()
    //             .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
    //         if let Some(pos) = world.get_component_mut::<Position>(player_entity) {
    //             *pos = player_pos;
    //         }
    //     }

    //     game.parallel_state.map.grid = vec![vec![Tile::new('.', "plains".to_string()); 100]; 100];
    //     game.parallel_state.map.grid[tree_pos.y as usize][tree_pos.x as usize].tile_type = 'T';

    //     let visible_tiles = game.get_visible_tiles(&player_pos, true);
    //     let visible_positions: std::collections::HashSet<Position> =
    //         visible_tiles.into_iter().map(|(p, _)| p).collect();

    //     assert!(
    //         visible_positions.contains(&tree_pos),
    //         "Tree should be visible"
    //     );
    //     assert!(
    //         !visible_positions.contains(&target_pos),
    //         "Tile behind tree should not be visible"
    //     );

    //     game.parallel_state.map.grid[tree_pos.y as usize][tree_pos.x as usize].tile_type = '.';

    //     let visible_tiles_no_tree = game.get_visible_tiles(&pos, true);
    //     let visible_positions_no_tree: std::collections::HashSet<Position> =
    //         visible_tiles_no_tree.into_iter().map(|(p, _)| p).collect();

    //     assert!(
    //         visible_positions_no_tree.contains(&target_pos),
    //         "Tile should be visible without tree"
    //     );
    //     Ok(())
    // }
}
