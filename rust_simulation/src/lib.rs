//! A simulation of a simple world with agents that can gather resources,
//! craft items, and build structures. The simulation is based on an
//! Entity-Component-System (ECS) architecture using `bevy_ecs`.

use bevy::prelude::*;
use rand::Rng;
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
pub mod state;
pub mod systems;
pub mod ui;
pub mod world;

use crate::state::AppState;
use crate::events::Event;
use animals::{pig::{Pig, SimpleAi}, wolf::{Wolf, WolfAI, wolf_ai_system}};
use components::{
    ai::{ExplorationFrontier, GoalQTable, KnownResources, MentalMap, PlayerMemories},
    status::{Health, Hunger},
    BrainComponent, Food, Inventory, Position, Velocity,
};
use config::*;
use item::ItemRegistry;
use map::Map;
use player::Player;
use recipes::RecipeManager;
use systems::ai::{actions, goal_selection, q_learning};
use systems::*;
use systems::visibility_system::visibility_system;
use crate::animals::wolf::WolfState;


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
    commands.init_resource::<async_task::AsyncResultChannel>();
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
            Food { value: 100.0 },
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

    // Spawn pigs
    let mut rng = rand::thread_rng();
    for _ in 0..config.pig_settings.num_pigs {
        let x = rng.gen_range(0..config.map_settings.width);
        let y = rng.gen_range(0..config.map_settings.height);
        commands.spawn((
            Pig,
            Position { x, y },
            Velocity { dx: 0, dy: 0 },
            Health {
                current: 50,
                max: 50,
            },
            Food { value: 25.0 },
            SimpleAi::default(),
        ));
    }


    // Spawn wolves
    for _ in 0..config.player_settings.num_wolves {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..config.map_settings.width);
        let y = rng.gen_range(0..config.map_settings.height);

        commands.spawn((
            Wolf,
            WolfAI::default(),
            Position { x, y },
            Velocity { dx: 0, dy: 0 },
            Health {
                current: 100,
                max: 100,
            },
            Hunger {
                current: 0.0,
                max: 100.0,
            },
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

pub fn wolf_eating_system(
    mut event_reader: EventReader<Event>,
    mut wolf_query: Query<&mut Hunger, With<Wolf>>,
    food_query: Query<&Food>,
) {
    for event in event_reader.read() {
        if let Event::EntityDied { entity, attacker } = event {
            if let Some(attacker_entity) = attacker {
                if let Ok(mut hunger) = wolf_query.get_mut(*attacker_entity) {
                    if let Ok(food) = food_query.get(*entity) {
                        hunger.current -= food.value;
                        if hunger.current < 0.0 {
                            hunger.current = 0.0;
                        }
                    }
                }
            }
        }
    }
}

pub fn add_simulation_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            update_day_night,
            systems::hunger::hunger_system,
            wolf_eating_system,
            wolf_ai_system,
            movement::movement_system,
            combat::combat_system,
            death::death_system,
            visibility_system,
        )
            .in_set(SimulationSet::Logic)
            .run_if(in_state(AppState::InGame)),
    );
}
