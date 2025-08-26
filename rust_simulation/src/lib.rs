//! A simulation of a simple world with agents that can gather resources,
//! craft items, and build structures. The simulation is based on an
//! Entity-Component-System (ECS) architecture using `bevy_ecs`.

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
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
use systems::ai::{actions, goal_selection, q_learning};
use systems::*;

// --- System Sets ---
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    Setup,
    Logic,
}

// --- Resources ---

#[derive(Resource)]
pub struct IsDay(pub bool);

#[derive(Resource)]
pub struct TickCount(pub u32);

#[derive(Resource)]
pub struct RecipeManagerResource(pub Arc<RecipeManager>);

#[derive(Resource)]
pub struct ItemRegistryResource(pub Arc<ItemRegistry>);

#[derive(Resource)]
pub struct DataPaths {
    pub biomes: String,
    pub resources: String,
    pub items: String,
    pub recipes: String,
}

// --- Simulation Setup ---

use std::fs;

use crate::systems::monitoring::MemoryLimitReached;

pub fn setup_simulation(
    mut commands: Commands,
    paths: Res<DataPaths>,
    config: Res<Config>,
    memory_limit_reached: Res<MemoryLimitReached>,
) {
    let map = Map::new(
        config.map_settings.width,
        config.map_settings.height,
        &paths.biomes,
        &paths.resources,
    )
    .unwrap();
    let item_registry = Arc::new(ItemRegistry::new(&paths.items).unwrap());
    let recipe_manager = Arc::new(RecipeManager::new(&paths.recipes).unwrap());

    // Load Q-tables if they exist
    let q_tables: HashMap<u32, GoalQTable> = if let Ok(data) = fs::read_to_string("q_tables.json") {
        serde_json::from_str(&data).unwrap_or_default()
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
            MentalMap(vec![
                vec![None; config.map_settings.width as usize];
                config.map_settings.height as usize
            ]),
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

pub fn add_simulation_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            update_day_night,
            systems::visibility_system::visibility_system,
            q_learning::update_q_table_system,
            goal_selection::goal_selection_system,
            actions::craft::craft_action_system,
            actions::attack::attack_action_system,
            actions::flee::flee_action_system,
            actions::explore::explore_action_system,
        )
            .chain()
            .in_set(SimulationSet::Logic),
    );

    app.add_systems(
        FixedUpdate,
        (
            actions::stockpile::stockpile_action_system,
            systems::pathfinding_system::pathfinding_system,
            systems::pathfinding_completion_system::pathfinding_completion_system,
            systems::path_movement_system::path_movement_system,
            movement::movement_system,
            find_resource::find_resource_system,
            gathering::gathering_system,
            crafting::crafting_system,
        )
            .chain()
            .in_set(SimulationSet::Logic),
    );

    app.add_systems(
        FixedUpdate,
        (
            building_logic::check_resources_system,
            building_logic::check_tile_system,
            building_logic::build_system,
            storage::storage_system,
            combat::combat_system,
            death::death_system,
        )
            .chain()
            .in_set(SimulationSet::Logic),
    );

    app.add_systems(
        FixedUpdate,
        (
            goal_selection::goal_planning_system,
            goal_selection::intent_creation_system,
        )
            .chain()
            .in_set(SimulationSet::Logic),
    );
    app.add_systems(
        FixedUpdate,
        (map_modification::map_modification_system)
            .in_set(SimulationSet::Logic),
    );
}
