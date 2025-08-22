//! A simulation of a simple world with agents that can gather resources,
//! craft items, and build structures. The simulation is based on an
//! Entity-Component-System (ECS) architecture using `bevy_ecs`.

pub mod async_task;
pub mod brain;
pub mod components;
pub mod config;
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

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{apply_deferred, ScheduleLabel};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use components::{
    ai::{ExplorationFrontier, GoalQTable, KnownResources, MentalMap, PlayerMemories},
    BrainComponent, Health, Inventory, Position, Equipped,
};
use config::*;
use errors::SimulationError;
use item::ItemRegistry;
use map::Map;
pub use player::Player;
use recipes::RecipeManager;
use systems::ai::{actions, goal_selection, q_learning};
use systems::*;

// --- Resources ---

#[derive(Resource)]
pub struct IsDay(pub bool);

#[derive(Resource)]
pub struct TickCount(pub u32);

#[derive(Resource)]
pub struct RecipeManagerResource(pub Arc<RecipeManager>);

#[derive(Resource)]
pub struct ItemRegistryResource(pub Arc<ItemRegistry>);


// --- Simulation Setup ---

pub fn setup_world(
    biomes_path: &str,
    resources_path: &str,
    items_path: &str,
    recipes_path: &str,
) -> Result<World, SimulationError> {
    let mut world = World::new();

    let map = Map::new(WIDTH, HEIGHT, biomes_path, resources_path)?;
    let item_registry = Arc::new(ItemRegistry::new(items_path)?);
    let recipe_manager = Arc::new(RecipeManager::new(recipes_path)?);

    world.insert_resource(map);
    world.insert_resource(ItemRegistryResource(Arc::clone(&item_registry)));
    world.insert_resource(RecipeManagerResource(Arc::clone(&recipe_manager)));
    world.insert_resource(IsDay(true));
    world.insert_resource(TickCount(0));
    world.init_resource::<async_task::AsyncResultChannel>();
    world.init_resource::<Events<events::Event>>();

    for i in 0..NUM_PLAYERS {
        world.spawn((
            Player::new(i, WIDTH, HEIGHT),
            Position { x: 0, y: 0 },
            Health { current: 100, max: 100 },
            Inventory::new(),
            Equipped { tool: None },
            BrainComponent::new(
                Arc::clone(&recipe_manager),
                Arc::clone(&item_registry),
                LEARNING_RATE,
                DISCOUNT_FACTOR,
                EPSILON,
            ),
            MentalMap(Arc::new(vec![vec![None; WIDTH as usize]; HEIGHT as usize])),
            KnownResources(HashMap::new()),
            PlayerMemories(HashMap::new()),
            GoalQTable(HashMap::new()),
            ExplorationFrontier(VecDeque::new()),
        ));
    }

    Ok(world)
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MySchedule {
    Main,
    Test,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum SimulationSet {
    AI,
    AsyncDispatch,
    ResultCollection,
    Physics,
    SyncActions,
    Finalize,
}

pub fn update_day_night(mut is_day: ResMut<IsDay>, mut tick_count: ResMut<TickCount>) {
    tick_count.0 += 1;
    is_day.0 = (tick_count.0 % (DAY_LENGTH + NIGHT_LENGTH)) < DAY_LENGTH;
}

pub fn create_schedule() -> Schedule {
    let mut schedule = Schedule::new(MySchedule::Main);

    schedule.configure_sets(
        (
            SimulationSet::AI,
            SimulationSet::AsyncDispatch,
            SimulationSet::ResultCollection,
            SimulationSet::Physics,
            SimulationSet::SyncActions,
            SimulationSet::Finalize,
        )
            .chain(),
    );

    schedule.add_systems(
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
            actions::equip::equip_action_system,
            systems::gathering::gathering_movement_system,
            systems::ai::pathfinding_failure::handle_pathfinding_failure_system,
        )
            .in_set(SimulationSet::AI),
    );

    schedule.add_systems(
        (
            systems::pathfinding_system::pathfinding_dispatcher_system,
            crafting::crafting_dispatcher_system,
            systems::gathering::gathering_dispatcher_system,
        )
            .in_set(SimulationSet::AsyncDispatch),
    );

    schedule.add_systems(
        (
            systems::async_result_collection_system::async_result_collection_system,
            apply_deferred,
        )
            .in_set(SimulationSet::ResultCollection),
    );

    schedule.add_systems(
        (
            systems::path_movement_system::path_movement_system,
            apply_deferred,
            movement::movement_system,
        )
            .in_set(SimulationSet::Physics),
    );

    schedule.add_systems(
        (
            building::building_system,
            storage::storage_system,
            combat::combat_system,
            equip::equip_system,
            throwing::throwing_system,
        )
            .in_set(SimulationSet::SyncActions),
    );

    schedule.add_systems(death::death_system.in_set(SimulationSet::Finalize));

    schedule
}

pub struct Game {
    pub world: World,
    schedule: Schedule,
    pub road_manager: road_manager::RoadManager,
}

impl Game {
    pub fn new(
        biomes_path: &str,
        resources_path: &str,
        items_path: &str,
        recipes_path: &str,
    ) -> Result<Self, SimulationError> {
        let world = setup_world(biomes_path, resources_path, items_path, recipes_path)?;
        let schedule = create_schedule();
        let road_manager = road_manager::RoadManager::new();
        Ok(Game {
            world,
            schedule,
            road_manager,
        })
    }

    pub fn tick(&mut self) -> Result<(), SimulationError> {
        self.schedule.run(&mut self.world);
        Ok(())
    }

    pub fn is_day(&self) -> bool {
        self.world.get_resource::<IsDay>().unwrap().0
    }

    pub fn tick_count(&self) -> u32 {
        self.world.get_resource::<TickCount>().unwrap().0
    }

    pub fn new_generation(&mut self) -> Result<(), SimulationError> {
        Ok(())
    }
}


