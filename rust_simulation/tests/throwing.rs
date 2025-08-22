mod common;

use bevy_ecs::prelude::*;
use common::create_test_world;
use rust_simulation::{
    components::{
        intents::WantsToThrow, DroppedItem, Health, Inventory, Position,
    },
    Player, MySchedule,
};

#[test]
fn test_throw_hits_target() {
    let mut world = create_test_world().unwrap();

    // Create thrower and target
    let thrower_entity = world
        .spawn((
            Position { x: 0, y: 0 },
            Player { _held_item: None },
            Inventory::new(),
            Health { current: 100, max: 100 },
        ))
        .id();
    let target_entity = world
        .spawn((
            Position { x: 1, y: 0 },
            Player { _held_item: None },
            Health { current: 100, max: 100 },
        ))
        .id();

    // Give thrower a "stone" to throw
    let mut inventory = world.get_mut::<Inventory>(thrower_entity).unwrap();
    inventory.add_item("stone", 1);
    drop(inventory);

    // Add the intent to throw
    world.entity_mut(thrower_entity).insert(WantsToThrow {
        target: target_entity,
        item_name: "stone".to_string(),
    });

    // Run the system.
    // Note: This test is probabilistic due to MISS_CHANCE. A better test
    // would inject a seeded RNG. For now, we run it multiple times.
    let mut hit_occured = false;
    for _ in 0..20 {
        let mut schedule = Schedule::new(MySchedule::Test);
        schedule.add_systems(rust_simulation::systems::throwing::throwing_system);
        schedule.run(&mut world);

        let health = world.get::<Health>(target_entity).unwrap();
        if health.current < 100 {
            hit_occured = true;
            break;
        }

        // Reset for next attempt if it missed
        world.get_mut::<Health>(target_entity).unwrap().current = 100;
        if let Some(mut inventory) = world.get_mut::<Inventory>(thrower_entity) {
            inventory.add_item("stone", 1);
        }
        world.entity_mut(thrower_entity).insert(WantsToThrow {
            target: target_entity,
            item_name: "stone".to_string(),
        });
    }

    assert!(hit_occured, "The throw should have hit the target at least once.");

    // Check that item was consumed
    let inventory = world.get::<Inventory>(thrower_entity).unwrap();
    assert!(!inventory.has_item("stone", 1));
}

#[test]
fn test_throw_misses_and_stacks() {
    let mut world = create_test_world().unwrap();

    let thrower_entity = world.spawn((Position { x: 50, y: 50 }, Player { _held_item: None }, Inventory::new())).id();
    let target_entity = world.spawn((Position { x: 0, y: 0 }, Player { _held_item: None }, Health { current: 100, max: 100 })).id();

    // Pre-existing dropped item
    world.spawn((
        Position { x: 0, y: 0 },
        DroppedItem { item_name: "stone".to_string(), quantity: 1 }
    ));

    let mut inventory = world.get_mut::<Inventory>(thrower_entity).unwrap();
    inventory.add_item("stone", 1);
    drop(inventory);

    // Force a miss by being out of range
    world.entity_mut(thrower_entity).insert(WantsToThrow {
        target: target_entity,
        item_name: "stone".to_string(),
    });

    let mut schedule = Schedule::new(MySchedule::Test);
    schedule.add_systems(rust_simulation::systems::throwing::throwing_system);
    schedule.run(&mut world);

    // Check that target was not damaged
    let health = world.get::<Health>(target_entity).unwrap();
    assert_eq!(health.current, 100);

    // Check that item stacked on the ground
    let mut found_stack = false;
    for (pos, dropped_item) in world.query::<(&Position, &DroppedItem)>().iter(&world) {
        if dropped_item.item_name == "stone" && pos.x == 0 && pos.y == 0 {
            assert_eq!(dropped_item.quantity, 2);
            found_stack = true;
            break;
        }
    }
    assert!(found_stack, "Item stack not found or not incremented.");
}
