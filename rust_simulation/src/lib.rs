//! A simulation of a simple world with agents that can gather resources,
//! craft items, and build structures. The simulation is based on an
//! Entity-Component-System (ECS) architecture using `bevy_ecs`.

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

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
pub mod systems;
pub mod world;

use components::{
    BrainComponent, Health, Inventory, Position,
    ai::{ExplorationFrontier, GoalQTable, KnownResources, MentalMap, PlayerMemories},
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

pub fn setup_simulation(mut commands: Commands, paths: Res<DataPaths>) {
    let map = Map::new(WIDTH, HEIGHT, &paths.biomes, &paths.resources).unwrap();
    let item_registry = Arc::new(ItemRegistry::new(&paths.items).unwrap());
    let recipe_manager = Arc::new(RecipeManager::new(&paths.recipes).unwrap());

    commands.insert_resource(map);
    commands.insert_resource(ItemRegistryResource(item_registry));
    commands.insert_resource(RecipeManagerResource(Arc::clone(&recipe_manager)));
    commands.insert_resource(IsDay(true));
    commands.insert_resource(TickCount(0));
    commands.init_resource::<async_task::AsyncResultChannel>();
    commands.init_resource::<Events<events::Event>>();

    for i in 0..NUM_PLAYERS {
        commands.spawn((
            Player::new(i, WIDTH, HEIGHT),
            Position { x: 0, y: 0 },
            Health {
                current: 100,
                max: 100,
            },
            Inventory::new(),
            BrainComponent::new(
                Arc::clone(&recipe_manager),
                LEARNING_RATE,
                DISCOUNT_FACTOR,
                EPSILON,
            ),
            MentalMap(vec![vec![None; WIDTH as usize]; HEIGHT as usize]),
            KnownResources(HashMap::new()),
            PlayerMemories(HashMap::new()),
            GoalQTable(HashMap::new()),
            ExplorationFrontier(VecDeque::new()),
        ));
    }
}

fn update_day_night(mut is_day: ResMut<IsDay>, mut tick_count: ResMut<TickCount>) {
    tick_count.0 += 1;
    is_day.0 = (tick_count.0 % (DAY_LENGTH + NIGHT_LENGTH)) < DAY_LENGTH;
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
            actions::stockpile::stockpile_action_system,
            systems::pathfinding_system::pathfinding_system,
            systems::async_result_collection_system::async_result_collection_system,
            systems::path_movement_system::path_movement_system,
            movement::movement_system,
            gathering::gathering_system,
            crafting::crafting_system,
            building::building_system,
            storage::storage_system,
            combat::combat_system,
            death::death_system,
        )
            .chain()
            .in_set(SimulationSet::Logic),
    );
}

