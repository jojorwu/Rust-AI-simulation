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
    BrainComponent, Health, Inventory, Position,
};
use config::*;
use errors::SimulationError;
use item::ItemRegistry;
use map::Map;
use player::Player;
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
    world.insert_resource(ItemRegistryResource(item_registry));
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
            BrainComponent::new(
                Arc::clone(&recipe_manager),
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

fn update_day_night(mut is_day: ResMut<IsDay>, mut tick_count: ResMut<TickCount>) {
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
            systems::gathering::gathering_movement_system,
        )
            .in_set(SimulationSet::AI),
    );

    schedule.add_systems(
        (
            systems::pathfinding_system::pathfinding_system,
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


// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use crate::components::{WantsToCraft, Resource as ResourceComponent};
    use crate::components::intents::IntendsToGather;
    use crate::components::path::{PathRequest, CurrentPath};


    fn create_test_world() -> Result<World, SimulationError> {
        let _ = env_logger::try_init();
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| SimulationError::EnvVarError(e.to_string()))?;
        setup_world(
            &format!("{manifest_dir}/data/biomes.json"),
            &format!("{manifest_dir}/data/resources.json"),
            &format!("{manifest_dir}/data/items.json"),
            &format!("{manifest_dir}/data/recipes.json"),
        )
    }

    #[test]
    fn test_pathfinding_flow() {
        let mut world = create_test_world().unwrap();
        let player_entity = world.query_filtered::<Entity, With<Player>>().iter(&world).next().unwrap();

        let mut map = world.get_resource_mut::<Map>().unwrap();
        map.set_tile(1, 0, crate::map::Tile::new('.', "grassland".to_string()));
        drop(map);

        world.entity_mut(player_entity).insert(PathRequest {
            start: (0, 0),
            goal: (1, 0),
        });

        let mut schedule = Schedule::new(MySchedule::Test);
        schedule.add_systems(systems::pathfinding_system::pathfinding_system);
        schedule.add_systems(systems::async_result_collection_system::async_result_collection_system);
        schedule.add_systems(apply_deferred);

        let mut path_found = false;
        for _ in 0..10 {
            schedule.run(&mut world);
            if world.get::<CurrentPath>(player_entity).is_some() {
                path_found = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(path_found, "Path was not found after timeout");
    }

    #[test]
    fn test_async_crafting_flow() {
        let mut world = create_test_world().unwrap();
        let player_entity = world.query_filtered::<Entity, With<Player>>().iter(&world).next().unwrap();

        let mut inventory = world.get_mut::<Inventory>(player_entity).unwrap();
        inventory.add_item("wood", 10);
        inventory.add_item("stone", 10);
        drop(inventory);

        world.entity_mut(player_entity).insert(WantsToCraft { item_name: "stone_axe".to_string() });

        let mut schedule = Schedule::new(MySchedule::Test);
        schedule.add_systems(crafting::crafting_dispatcher_system);
        schedule.add_systems(systems::async_result_collection_system::async_result_collection_system);

        let mut craft_complete = false;
        for _ in 0..10 {
            schedule.run(&mut world);
            let inventory = world.get::<Inventory>(player_entity).unwrap();
            if inventory.has_item("stone_axe", 1) {
                craft_complete = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(craft_complete, "Player should have a stone axe after crafting");
    }

    #[test]
    fn test_async_gathering_flow() {
        let mut world = create_test_world().unwrap();
        let player_entity = world.query_filtered::<Entity, With<Player>>().iter(&world).next().unwrap();

        world.spawn((
            Position { x: 1, y: 0 },
            ResourceComponent { name: "wood".to_string(), quantity: 5 }
        ));

        world.entity_mut(player_entity).insert(IntendsToGather("wood".to_string()));

        let mut schedule = Schedule::new(MySchedule::Test);
        schedule.add_systems(gathering::gathering_dispatcher_system);
        schedule.add_systems(systems::async_result_collection_system::async_result_collection_system);

        let mut gather_complete = false;
        for _ in 0..10 {
            schedule.run(&mut world);
            let inventory = world.get::<Inventory>(player_entity).unwrap();
            if inventory.has_item("wood", 1) {
                gather_complete = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(gather_complete, "Player should have wood after gathering");
    }
}
