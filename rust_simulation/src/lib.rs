//! A simulation of a simple world with agents that can gather resources,
//! craft items, and build structures. The simulation is based on an
//! Entity-Component-System (ECS) architecture using `bevy_ecs`.

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
use bevy_ecs::schedule::ScheduleLabel;
use std::sync::Arc;

use components::{BrainComponent, Inventory, Position, Health};
use config::*;
use errors::SimulationError;
use item::ItemRegistry;
use map::Map;
use player::Player;
use recipes::RecipeManager;
use systems::ai::{action_execution, goal_selection, q_learning};
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

/// Creates and configures the main Bevy ECS `World`.
pub fn setup_world(
    biomes_path: &str,
    resources_path: &str,
    items_path: &str,
    recipes_path: &str,
) -> Result<World, SimulationError> {
    let mut world = World::new();

    // Insert resources
    let map = Map::new(WIDTH, HEIGHT, biomes_path, resources_path)?;
    let item_registry = Arc::new(ItemRegistry::new(items_path)?);
    let recipe_manager = Arc::new(RecipeManager::new(recipes_path)?);

    world.insert_resource(map);
    world.insert_resource(ItemRegistryResource(item_registry));
    world.insert_resource(RecipeManagerResource(Arc::clone(&recipe_manager)));
    world.insert_resource(IsDay(true));
    world.insert_resource(TickCount(0));

    // Initialize events
    world.init_resource::<Events<events::Event>>();

    // Spawn initial entities (players)
    for i in 0..NUM_PLAYERS {
        world.spawn((
            Player::new(i, WIDTH, HEIGHT),
            Position { x: 0, y: 0 }, // Position will be updated later
            Health { current: 100, max: 100 },
            Inventory::new(),
            BrainComponent::new(
                Arc::clone(&recipe_manager),
                LEARNING_RATE,
                DISCOUNT_FACTOR,
                EPSILON,
            ),
        ));
    }

    // TODO: Set initial player positions correctly
    // TODO: Spawn initial resources on the map

    Ok(world)
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MySchedule {
    Main,
    Test,
}

fn update_day_night(mut is_day: ResMut<IsDay>, mut tick_count: ResMut<TickCount>) {
    tick_count.0 += 1;
    is_day.0 = (tick_count.0 % (DAY_LENGTH + NIGHT_LENGTH)) < DAY_LENGTH;
}

/// Creates the main schedule for the simulation.
pub fn create_schedule() -> Schedule {
    let mut schedule = Schedule::new(MySchedule::Main);

    // Add systems to the schedule
    // Note: These systems need to be refactored to be Bevy systems.
    schedule
        .add_systems(update_day_night)
        .add_systems(q_learning::update_q_table_system)
        .add_systems(goal_selection::goal_selection_system)
        .add_systems(action_execution::action_execution_system)
        .add_systems(gathering::gathering_system)
        .add_systems(crafting::crafting_system)
        .add_systems(building::building_system)
        .add_systems(storage::storage_system)
        .add_systems(combat::combat_system)
        .add_systems(death::death_system);

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

    fn create_test_world() -> Result<World, SimulationError> {
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
    fn test_world_setup() -> Result<(), SimulationError> {
        let mut world = create_test_world()?;
        assert_eq!(world.query::<&Player>().iter(&world).count(), NUM_PLAYERS as usize);

        let map = world.get_resource::<Map>().unwrap();
        assert_eq!(map.width, WIDTH);
        assert_eq!(map.height, HEIGHT);

        let recipe_manager = world.get_resource::<RecipeManagerResource>().unwrap();
        assert!(recipe_manager.0.recipes.contains_key("stone_axe"));

        assert_eq!(
            world.query::<&BrainComponent>().iter(&world).count(),
            NUM_PLAYERS as usize
        );

        Ok(())
    }

    #[test]
    fn test_day_night_cycle() {
        let mut world = create_test_world().unwrap();
        let mut schedule = Schedule::new(MySchedule::Test);

        schedule.add_systems(update_day_night);

        // Initial state
        assert!(world.get_resource::<IsDay>().unwrap().0);
        assert_eq!(world.get_resource::<TickCount>().unwrap().0, 0);

        // Test day -> day
        schedule.run(&mut world);
        assert!(world.get_resource::<IsDay>().unwrap().0);
        assert_eq!(world.get_resource::<TickCount>().unwrap().0, 1);

        // Test day -> night transition
        world.get_resource_mut::<TickCount>().unwrap().0 = DAY_LENGTH - 1;
        schedule.run(&mut world); // Tick becomes DAY_LENGTH
        assert!(!world.get_resource::<IsDay>().unwrap().0);
        assert_eq!(world.get_resource::<TickCount>().unwrap().0, DAY_LENGTH);

        // Test night -> night
        schedule.run(&mut world);
        assert!(!world.get_resource::<IsDay>().unwrap().0);
        assert_eq!(world.get_resource::<TickCount>().unwrap().0, DAY_LENGTH + 1);

        // Test night -> day transition
        world.get_resource_mut::<TickCount>().unwrap().0 = DAY_LENGTH + NIGHT_LENGTH - 1;
        schedule.run(&mut world); // Tick becomes DAY_LENGTH + NIGHT_LENGTH
        assert!(world.get_resource::<IsDay>().unwrap().0);
    }
}
