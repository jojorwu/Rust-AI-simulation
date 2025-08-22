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
pub mod pathfinding_async;
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
    world.init_resource::<pathfinding_async::PathfindingResultChannel>();

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
            MentalMap(vec![vec![None; WIDTH as usize]; HEIGHT as usize]),
            KnownResources(HashMap::new()),
            PlayerMemories(HashMap::new()),
            GoalQTable(HashMap::new()),
            ExplorationFrontier(VecDeque::new()),
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
        .add_systems(systems::visibility_system::visibility_system)
        .add_systems(q_learning::update_q_table_system)
        .add_systems(goal_selection::goal_selection_system)
        .add_systems(actions::craft::craft_action_system)
        .add_systems(actions::attack::attack_action_system)
        .add_systems(actions::flee::flee_action_system)
        .add_systems(actions::explore::explore_action_system)
        .add_systems(actions::stockpile::stockpile_action_system)
        .add_systems(systems::pathfinding_system::pathfinding_system)
        .add_systems(systems::path_collection_system::path_collection_system)
        .add_systems(apply_deferred)
        .add_systems(systems::path_movement_system::path_movement_system)
        .add_systems(apply_deferred) // Flush Velocity commands
        .add_systems(movement::movement_system)
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

        // Run visibility system once and check that it has an effect
        let mut schedule = create_schedule();
        schedule.run(&mut world);

        let mut player_query = world.query::<(Entity, &MentalMap, &ExplorationFrontier)>();
        let (_, mental_map, exploration_frontier) = player_query.iter(&world).next().unwrap();

        // Check that the mental map has been updated
        let is_mental_map_updated = mental_map.0.iter().any(|row| row.iter().any(|tile| tile.is_some()));
        assert!(is_mental_map_updated, "Mental map was not updated after running visibility system");

        // Check that the exploration frontier has been populated
        assert!(!exploration_frontier.0.is_empty(), "Exploration frontier is empty after running visibility system");


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

    #[test]
    fn test_visibility_system_populates_memory() {
        let mut world = create_test_world().unwrap();

        let _player_entity = world.query_filtered::<Entity, With<Player>>().iter(&world).next().unwrap();

        let (mental_map, exploration_frontier) = world.query::<(&MentalMap, &ExplorationFrontier)>().iter(&world).next().unwrap();
        let is_mental_map_empty = mental_map.0.iter().all(|row| row.iter().all(|tile| tile.is_none()));
        assert!(is_mental_map_empty, "Mental map should be empty at the start");
        assert!(exploration_frontier.0.is_empty(), "Exploration frontier should be empty at the start");

        // Run the visibility system
        let mut schedule = Schedule::new(MySchedule::Test);
        schedule.add_systems(systems::visibility_system::visibility_system);
        schedule.run(&mut world);

        // Ensure map and frontier are now populated
        let (mental_map, exploration_frontier) = world.query::<(&MentalMap, &ExplorationFrontier)>().iter(&world).next().unwrap();
        let is_mental_map_updated = mental_map.0.iter().any(|row| row.iter().any(|tile| tile.is_some()));
        assert!(is_mental_map_updated, "Mental map was not updated after running visibility system");
        assert!(!exploration_frontier.0.is_empty(), "Exploration frontier is empty after running visibility system");
    }

    #[test]
    fn test_goal_selection_flee_on_low_health() {
        use components::intents::IntendsToFlee;

        let mut world = create_test_world().unwrap();
        let player_entity = world.query_filtered::<Entity, With<Player>>().iter(&world).next().unwrap();

        // Set player health to a low value
        let mut health = world.get_mut::<Health>(player_entity).unwrap();
        health.current = 1;

        // Run the goal selection system
        let mut schedule = Schedule::new(MySchedule::Test);
        schedule.add_systems(goal_selection::goal_selection_system);
        schedule.run(&mut world);

        // Check that the agent now intends to flee
        let player_has_flee_intent = world.get::<IntendsToFlee>(player_entity).is_some();
        assert!(player_has_flee_intent, "Agent should have IntendsToFlee component on low health");
    }

    #[test]
    fn test_craft_action_flow() {
        use components::intents::IntendsToCraft;
        use components::WantsToCraft;

        let mut world = create_test_world().unwrap();
        let player_entity = world.query_filtered::<Entity, With<Player>>().iter(&world).next().unwrap();

        // Manually add the intent
        world.entity_mut(player_entity).insert(IntendsToCraft("stone_axe".to_string()));

        // Run the craft action system
        let mut schedule = Schedule::new(MySchedule::Test);
        schedule.add_systems(actions::craft::craft_action_system);
        schedule.run(&mut world);

        // Check that the agent now wants to craft the item
        let wants_to_craft = world.get::<WantsToCraft>(player_entity);
        assert!(wants_to_craft.is_some(), "Player should have WantsToCraft component");
        assert_eq!(wants_to_craft.unwrap().item_name, "stone_axe");

        // Check that the intent component was removed
        let intent_was_removed = world.get::<IntendsToCraft>(player_entity).is_none();
        assert!(intent_was_removed, "IntendsToCraft component should be removed after system runs");
    }

    #[test]
    fn test_pathfinding_flow() {
        use crate::map;
        use crate::systems::movement::movement_system;
        use components::path::{PathRequest, CurrentPath};

        let mut world = create_test_world().unwrap();

        // Manually make the area around the player walkable for a deterministic test.
        let map = world.get_resource_mut::<Map>().unwrap();
        for y in 0..10 {
            for x in 0..10 {
                map.set_tile(x, y, map::Tile::new('.', "plains".to_string()));
            }
        }
        drop(map);

        let player_entity = world.query_filtered::<Entity, With<Player>>().iter(&world).next().unwrap();

        // Run visibility once to populate the mental map
        let mut vis_schedule = Schedule::new(MySchedule::Test);
        vis_schedule.add_systems(systems::visibility_system::visibility_system);
        vis_schedule.run(&mut world);

        // 1. Get a valid goal from the frontier
        let goal_pos = {
            let exploration_frontier = world.query::<&ExplorationFrontier>().iter(&world).next().unwrap();
            assert!(!exploration_frontier.0.is_empty(), "Frontier should not be empty after visibility run");
            exploration_frontier.0.front().unwrap().clone()
        };

        // 2. Add a PathRequest to that goal
        world.entity_mut(player_entity).insert(PathRequest {
            start: (0, 0),
            goal: (goal_pos.x, goal_pos.y),
        });

        // 2. Run the systems
        let mut schedule = Schedule::new(MySchedule::Test);
        schedule.add_systems(systems::pathfinding_system::pathfinding_system);
        schedule.add_systems(systems::path_collection_system::path_collection_system);
        schedule.add_systems(apply_deferred);
        schedule.add_systems(systems::path_movement_system::path_movement_system);
        schedule.add_systems(movement_system);

        // Tick until the path is found (or timeout)
        let mut path_found = false;
        for _ in 0..50 { // Increased timeout
            schedule.run(&mut world);
            if world.get::<CurrentPath>(player_entity).is_some() {
                path_found = true;
                break;
            }
            // Give the background thread some time to work.
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(path_found, "Path was not found after timeout");
        assert!(world.get::<PathRequest>(player_entity).is_none(), "PathRequest should be removed");

        // Now that the path is found, check movement.
        // The first tick of movement should not change position due to deferred commands.
        let initial_pos = *world.get::<Position>(player_entity).unwrap();
        schedule.run(&mut world);
        assert_eq!(world.get::<Position>(player_entity).unwrap(), &initial_pos, "Position should not change on first movement tick");

        // The second tick of movement should change the position.
        schedule.run(&mut world);
        let new_pos = world.get::<Position>(player_entity).unwrap();
        assert_ne!(&initial_pos, new_pos, "Position should change on second movement tick");
    }
}
