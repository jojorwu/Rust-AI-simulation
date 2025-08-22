mod common;

use bevy_ecs::prelude::*;
use common::create_test_world;
use rust_simulation::{
    components::{
        intents::WantsToThrow, DroppedItem, Health, Inventory, Position,
    },
    Player, MySchedule, consts,
};

#[test]
fn test_throw_hit() {
    let mut world = create_test_world().unwrap();

    let thrower_entity = world.spawn((Position { x: 0, y: 0 }, Player { _held_item: None }, Inventory::new())).id();
    let target_entity = world.spawn((Position { x: 1, y: 0 }, Player { _held_item: None }, Health { current: 100, max: 100 })).id();

    world.get_mut::<Inventory>(thrower_entity).unwrap().add_item(consts::STONE, 1);

    world.entity_mut(thrower_entity).insert(WantsToThrow {
        target: target_entity,
        item_name: consts::STONE.to_string(),
    });

    let mut schedule = Schedule::new(MySchedule::Test);
    schedule.add_systems(rust_simulation::systems::throwing::throwing_system);

    // Run multiple times because of miss chance
    for _ in 0..20 {
        schedule.run(&mut world);
        let health = world.get::<Health>(target_entity).unwrap();
        if health.current < 100 {
            break;
        }
        // Reset state for next attempt
        if let Some(mut inv) = world.get_mut::<Inventory>(thrower_entity) {
            inv.add_item(consts::STONE, 1);
        }
        world.entity_mut(thrower_entity).insert(WantsToThrow {
            target: target_entity,
            item_name: consts::STONE.to_string(),
        });
    }

    let health = world.get::<Health>(target_entity).unwrap();
    assert!(health.current < 100, "Target should have taken damage");
}

#[test]
fn test_throw_miss_and_stacks() {
    let mut world = create_test_world().unwrap();

    // Place thrower far away to guarantee a miss due to range.
    let thrower_entity = world.spawn((Position { x: 50, y: 50 }, Player { _held_item: None }, Inventory::new())).id();
    let target_entity = world.spawn((Position { x: 0, y: 0 }, Player { _held_item: None }, Health { current: 100, max: 100 })).id();

    world.spawn((
        Position { x: 0, y: 0 },
        DroppedItem { item_name: consts::STONE.to_string(), quantity: 1 }
    ));
    world.get_mut::<Inventory>(thrower_entity).unwrap().add_item(consts::STONE, 1);

    world.entity_mut(thrower_entity).insert(WantsToThrow {
        target: target_entity,
        item_name: consts::STONE.to_string(),
    });

    let mut schedule = Schedule::new(MySchedule::Test);
    schedule.add_systems(rust_simulation::systems::throwing::throwing_system);
    schedule.run(&mut world);

    let health = world.get::<Health>(target_entity).unwrap();
    assert_eq!(health.current, 100, "Target should not have taken damage on a miss");

    let mut found_stack = false;
    for (pos, dropped_item) in world.query::<(&Position, &DroppedItem)>().iter(&world) {
        if dropped_item.item_name == consts::STONE && pos.x == 0 && pos.y == 0 {
            assert_eq!(dropped_item.quantity, 2, "Item should have stacked");
            found_stack = true;
            break;
        }
    }
    assert!(found_stack, "Item stack not found or not incremented.");
}
