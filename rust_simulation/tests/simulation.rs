mod common;

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::apply_deferred;
use common::create_test_world;
use rust_simulation::{
    components::{
        ai::*,
        intents::*,
        path::*,
        BrainComponent, Health, Inventory, Position, Resource as ResourceComponent, WantsToCraft,
    },
    Player,
    config::{DAY_LENGTH, NUM_PLAYERS},
    errors::SimulationError,
    systems::{self, ai::goal_selection},
    IsDay, MySchedule, TickCount,
};

#[test]
fn test_world_setup() -> Result<(), SimulationError> {
    let mut world = create_test_world()?;
    assert_eq!(
        world.query::<&Player>().iter(&world).count(),
        NUM_PLAYERS as usize
    );
    Ok(())
}

#[test]
fn test_day_night_cycle() {
    let mut world = create_test_world().unwrap();
    let mut schedule = Schedule::new(MySchedule::Test);
    schedule.add_systems(rust_simulation::update_day_night);
    assert!(world.get_resource::<IsDay>().unwrap().0);
    world.get_resource_mut::<TickCount>().unwrap().0 = DAY_LENGTH - 1;
    schedule.run(&mut world);
    assert!(!world.get_resource::<IsDay>().unwrap().0);
}

#[test]
fn test_goal_selection_flee_on_low_health() {
    let mut world = create_test_world().unwrap();
    let player_entity = world
        .query_filtered::<Entity, With<Player>>()
        .iter(&world)
        .next()
        .unwrap();

    let mut health = world.get_mut::<Health>(player_entity).unwrap();
    health.current = 1;

    let mut schedule = Schedule::new(MySchedule::Test);
    schedule.add_systems(goal_selection::goal_selection_system);
    schedule.run(&mut world);

    let player_has_flee_intent = world.get::<IntendsToFlee>(player_entity).is_some();
    assert!(
        player_has_flee_intent,
        "Agent should have IntendsToFlee component on low health"
    );
}

#[test]
fn test_craft_action_flow() {
    let mut world = create_test_world().unwrap();
    let player_entity = world
        .query_filtered::<Entity, With<Player>>()
        .iter(&world)
        .next()
        .unwrap();

    world
        .entity_mut(player_entity)
        .insert(IntendsToCraft("stone_axe".to_string()));

    let mut schedule = Schedule::new(MySchedule::Test);
    schedule.add_systems(systems::ai::actions::craft::craft_action_system);
    schedule.run(&mut world);

    let wants_to_craft = world.get::<WantsToCraft>(player_entity);
    assert!(
        wants_to_craft.is_some(),
        "Player should have WantsToCraft component"
    );
}

#[test]
fn test_pathfinding_failure_triggers_goal_reset() {
    let mut world = create_test_world().unwrap();
    let player_entity = world
        .query_filtered::<Entity, With<Player>>()
        .iter(&world)
        .next()
        .unwrap();

    let mut brain = world.get_mut::<BrainComponent>(player_entity).unwrap();
    brain.current_goal = Some(rust_simulation::brain::Goal::Explore);
    brain.goal_commitment_ticks = 10;
    drop(brain);

    world
        .entity_mut(player_entity)
        .insert(PathfindingFailure);

    let mut schedule = Schedule::new(MySchedule::Test);
    schedule.add_systems(systems::ai::pathfinding_failure::handle_pathfinding_failure_system);
    schedule.run(&mut world);

    assert!(
        world.get::<PathfindingFailure>(player_entity).is_none(),
        "PathfindingFailure should be removed"
    );

    let brain = world.get::<BrainComponent>(player_entity).unwrap();
    assert!(
        brain.current_goal.is_none(),
        "Goal should be reset on pathfinding failure"
    );
}

#[test]
fn test_pathfinding_flow() {
    let mut world = create_test_world().unwrap();
    let player_entity = world.query_filtered::<Entity, With<Player>>().iter(&world).next().unwrap();

    let mut map = world.get_resource_mut::<rust_simulation::map::Map>().unwrap();
    map.set_tile(1, 0, rust_simulation::map::Tile::new('.', "grassland".to_string()));
    drop(map);

    world.entity_mut(player_entity).insert(PathRequest {
        start: (0, 0),
        goal: (1, 0),
    });

    let mut schedule = Schedule::new(MySchedule::Test);
    schedule.add_systems(systems::pathfinding_system::pathfinding_dispatcher_system);
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
    schedule.add_systems(systems::crafting::crafting_dispatcher_system);
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
    schedule.add_systems(systems::gathering::gathering_dispatcher_system);
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
