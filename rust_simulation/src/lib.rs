//! # Rust Simulation Library
//!
//! This library contains the core logic for a simulation of a simple world
//! where agents can gather resources, craft items, and build structures.
//! It is built using the Bevy game engine's Entity-Component-System (ECS)
//! architecture.
//!
//! The main components of the simulation are:
//! - **Entities**: Agents, resources, items, etc.
//! - **Components**: Data associated with entities (e.g., `Position`, `Health`).
//! - **Systems**: Logic that operates on entities with specific components.
//! - **AI**: A Q-learning based decision-making system for agents.

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

pub mod animals;
pub mod async_task;
pub mod brain;
pub mod components;
pub mod config;
pub mod errors;
pub mod events;
pub mod fov;
pub mod graphics;
pub mod item;
pub mod map;
pub mod pathfinding;
pub mod player;
pub mod recipes;
pub mod road;
pub mod road_builder;
pub mod road_manager;
pub mod serde_helpers;
pub mod state;
pub mod systems;
pub mod ui;
pub mod world;

use components::{
    ai::{ExplorationFrontier, GoalQTable, KnownResources, MentalMap, PlayerMemories},
    status::Health,
    BrainComponent, Inventory, Position,
};
use config::*;
use item::ItemRegistry;
use map::Map;
use player::Player;
use recipes::RecipeManager;
use systems::ai::{actions, goal_completion, goal_selection, q_learning};
use systems::*;

// --- System Sets ---

/// Defines the primary system sets used for ordering in the simulation.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    /// Systems that run once at the beginning of the simulation.
    Setup,
    /// Systems that run on every tick of the `FixedUpdate` schedule.
    Logic,
}

// --- Resources ---

/// A resource that indicates whether it is currently day or night.
#[derive(Resource)]
pub struct IsDay(pub bool);

/// A resource that tracks the number of ticks that have passed.
#[derive(Resource)]
pub struct TickCount(pub u32);

/// A resource to hold the shared `RecipeManager`.
#[derive(Resource)]
pub struct RecipeManagerResource(pub Arc<RecipeManager>);

/// A resource to hold the shared `ItemRegistry`.
#[derive(Resource)]
pub struct ItemRegistryResource(pub Arc<ItemRegistry>);

/// A resource holding the paths to static data files.
#[derive(Resource)]
pub struct DataPaths {
    /// Path to `biomes.json`.
    pub biomes: String,
    /// Path to `resources.json`.
    pub resources: String,
    /// Path to `items.json`.
    pub items: String,
    /// Path to `recipes.json`.
    pub recipes: String,
}

/// A resource holding the paths to user-specific application directories.
#[derive(Resource)]
pub struct AppPaths {
    /// The directory where persistent data (like `q_tables.json`) is stored.
    pub data_dir: PathBuf,
}

// --- Simulation Setup ---

use std::fs;

use crate::systems::monitoring::MemoryLimitReached;

/// A Bevy startup system that sets up the initial state of the simulation.
///
/// This system is responsible for:
/// - Creating the game map.
/// - Loading registries for items and recipes.
/// - Loading saved AI state (Q-tables).
/// - Spawning the initial set of player agents.
pub fn setup_simulation(
    mut commands: Commands,
    paths: Res<DataPaths>,
    app_paths: Res<AppPaths>,
    config: Res<Config>,
    memory_limit_reached: Res<MemoryLimitReached>,
) {
    let map = Map::new(
        config.map_settings.width,
        config.map_settings.height,
        &paths.biomes,
        &paths.resources,
    )
    .expect("Failed to create Map");
    let item_registry = Arc::new(ItemRegistry::new(&paths.items).expect("Failed to create ItemRegistry"));
    let recipe_manager = Arc::new(RecipeManager::new(&paths.recipes).expect("Failed to create RecipeManager"));

    // Load Q-tables if they exist
    let q_table_path = app_paths.data_dir.join("q_tables.json");
    let q_tables: HashMap<u32, GoalQTable> = if let Ok(data) = fs::read_to_string(q_table_path) {
        match serde_json::from_str(&data) {
            Ok(tables) => tables,
            Err(e) => {
                log::warn!("Failed to parse q_tables.json: {e}. Starting with empty Q-tables.");
                HashMap::new()
            }
        }
    } else {
        HashMap::new()
    };

    commands.insert_resource(map);
    commands.insert_resource(ItemRegistryResource(item_registry));
    commands.insert_resource(RecipeManagerResource(Arc::clone(&recipe_manager)));
    commands.insert_resource(IsDay(true));
    commands.insert_resource(TickCount(0));
    commands.init_resource::<Events<events::Event>>();

    if memory_limit_reached.0 {
        log::warn!("RAM limit reached, not spawning any agents.");
        return;
    }

    for i in 0..config.player_settings.num_players {
        let q_table = q_tables
            .get(&i)
            .cloned()
            .unwrap_or_else(|| GoalQTable(HashMap::new()));

        commands.spawn((
            Player::new(i, config.map_settings.width, config.map_settings.height),
            Position { x: 0, y: 0 },
            Health {
                current: 100,
                max: 100,
            },
            Inventory::new(),
            BrainComponent::new(
                Arc::clone(&recipe_manager),
                config.ai.q_learning.learning_rate,
                config.ai.q_learning.discount_factor,
                config.ai.q_learning.epsilon,
            ),
            MentalMap(Arc::new(HashMap::new())),
            KnownResources(HashMap::new()),
            PlayerMemories(HashMap::new()),
            q_table,
            ExplorationFrontier(VecDeque::new()),
        ));
    }
}

fn update_day_night(
    mut is_day: ResMut<IsDay>,
    mut tick_count: ResMut<TickCount>,
    config: Res<Config>,
) {
    tick_count.0 += 1;
    let day_night_cycle = &config.day_night_cycle;
    is_day.0 = (tick_count.0 % (day_night_cycle.day_length + day_night_cycle.night_length))
        < day_night_cycle.day_length;
}

/// Adds all the simulation's systems to the Bevy `App`.
///
/// This function organizes the systems into a coherent execution order using
/// system sets and chaining to prevent race conditions and ensure logical consistency.
pub fn add_simulation_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            // --- Perception and State Updates ---
            // These systems update the agent's internal state and perception of the world.
            update_day_night,
            visibility_system::visibility_system,
            hunger::hunger_system,
            eating::eating_system,
            // --- AI Decision Making ---
            // This chain handles the core AI loop from goal selection to planning.
            (
                goal_selection::goal_selection_system,
                goal_selection::goal_planning_system,
                goal_selection::intent_creation_system,
            )
                .chain(),
            // --- Action Systems ---
            // These systems execute the intents produced by the AI.
            // They are grouped to run in parallel where possible.
            (
                actions::craft::craft_action_system,
                actions::attack::attack_action_system,
                actions::flee::flee_action_system,
                actions::explore::explore_action_system,
                actions::stockpile::stockpile_action_system,
                find_resource::find_resource_system,
                gathering::gathering_system,
                crafting::crafting_system,
                building_logic::check_resources_system,
                building_logic::build_system,
                storage::storage_system,
                combat::combat_system,
                death::death_system,
                map_modification::map_modification_system,
            ),
            // --- Pathfinding and Movement ---
            // This chain handles calculating and following paths.
            (
                pathfinding_system::pathfinding_system,
                pathfinding_completion_system::pathfinding_completion_system,
                path_movement_system::path_movement_system,
                movement::movement_system,
            )
                .chain(),
            // --- Goal Completion and Learning ---
            goal_completion::goal_completion_system,
            q_learning::update_q_table_system,
        )
            .in_set(SimulationSet::Logic),
    );
}
