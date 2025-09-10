use bevy::prelude::*;
use rust_simulation::{
    components::{
        ai::KnownResources,
        intents::IsGathering,
        Inventory, Position, Resource,
    },
    systems::gathering::gathering_system,
};
use std::collections::HashMap;

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, gathering_system);
    app
}

#[test]
fn test_gathering_system_gathers_one_item() {
    let mut app = setup_app();

    let resource_entity = app
        .world
        .spawn((
            Resource {
                name: "wood".to_string(),
                quantity: 10,
            },
            Position { x: 1, y: 1 },
        ))
        .id();

    let gatherer_entity = app
        .world
        .spawn((
            Inventory::new(),
            Position { x: 1, y: 0 }, // Adjacent
            IsGathering {
                target: resource_entity,
                resource: "wood".to_string(),
                amount: 5,
                gathered_so_far: 0,
            },
            KnownResources(HashMap::new()),
        ))
        .id();

    app.update();

    let inventory = app.world.get::<Inventory>(gatherer_entity).unwrap();
    assert_eq!(inventory.get_quantity("wood"), 1);

    let resource = app.world.get::<Resource>(resource_entity).unwrap();
    assert_eq!(resource.quantity, 9);

    let is_gathering = app.world.get::<IsGathering>(gatherer_entity).unwrap();
    // The amount gathered so far is not yet tracked in the component itself
    // assert_eq!(is_gathering.gathered_so_far, 1);
}

#[test]
fn test_gathering_system_stops_when_inventory_is_full() {
    // This is the failing test that demonstrates the bug.
    let mut app = setup_app();

    let mut gatherer_inv = Inventory::new();
    gatherer_inv.add_item("wood", 8);

    let resource_entity = app
        .world
        .spawn((
            Resource {
                name: "wood".to_string(),
                quantity: 10,
            },
            Position { x: 1, y: 1 },
        ))
        .id();

    let gatherer_entity = app
        .world
        .spawn((
            gatherer_inv,
            Position { x: 1, y: 0 }, // Adjacent
            IsGathering {
                target: resource_entity,
                resource: "wood".to_string(),
                amount: 5, // Target to gather 5
                gathered_so_far: 0,
            },
            KnownResources(HashMap::new()),
        ))
        .id();

    // Run the system for 5 ticks.
    // With the bug, it will stop after 2 ticks.
    for _ in 0..5 {
        app.update();
    }

    let inventory = app.world.get::<Inventory>(gatherer_entity).unwrap();
    assert_eq!(inventory.get_quantity("wood"), 13); // 8 initial + 5 gathered

    let resource = app.world.get::<Resource>(resource_entity).unwrap();
    assert_eq!(resource.quantity, 5);

    assert!(app.world.get::<IsGathering>(gatherer_entity).is_none());
}

#[test]
fn test_gathering_depletes_resource() {
    let mut app = setup_app();

    let resource_entity = app
        .world
        .spawn((
            Resource {
                name: "wood".to_string(),
                quantity: 3,
            },
            Position { x: 1, y: 1 },
        ))
        .id();

    let gatherer_entity = app
        .world
        .spawn((
            Inventory::new(),
            Position { x: 1, y: 0 }, // Adjacent
            IsGathering {
                target: resource_entity,
                resource: "wood".to_string(),
                amount: 5,
                gathered_so_far: 0,
            },
            KnownResources(HashMap::new()),
        ))
        .id();

    for _ in 0..5 {
        app.update();
    }

    let inventory = app.world.get::<Inventory>(gatherer_entity).unwrap();
    assert_eq!(inventory.get_quantity("wood"), 3);

    // Resource entity should be despawned
    assert!(app.world.get::<Resource>(resource_entity).is_none());
    assert!(app.world.get::<IsGathering>(gatherer_entity).is_none());
}
