use rust_simulation::{
    components::{
        ai::KnownResources,
        intents::{IntendsToGather, IsGathering},
        Inventory, Position, Resource as ResourceComponent,
    },
    map::Map,
    systems::{find_resource::find_resource_system, gathering::gathering_system},
};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[test]
fn test_find_resource_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let map = Map::new(10, 10, "data/biomes.json", "data/resources.json")
        .expect("Failed to create map");
    let resource_pos = Position { x: 5, y: 5 };
    let resource_entity = app
        .world
        .spawn((
            rust_simulation::components::Resource {
                name: "wood".to_string(),
                quantity: 10,
            },
            resource_pos,
        ))
        .id();
    map.add_entity_to_spatial_map(resource_entity, 5, 5);
    app.insert_resource(map);

    let mut known_resources = KnownResources(HashMap::new());
    let mut positions = HashSet::new();
    positions.insert(resource_pos);
    known_resources
        .0
        .insert("wood".to_string(), positions);
    let gatherer_entity = app
        .world
        .spawn((
            known_resources,
            Position { x: 0, y: 0 },
            IntendsToGather("wood".to_string(), 1),
        ))
        .id();

    app.add_systems(Update, find_resource_system);
    app.update();

    let gatherer = app.world.entity(gatherer_entity);
    let is_gathering = gatherer
        .get::<IsGathering>()
        .expect("Gatherer should have IsGathering component");
    assert_eq!(is_gathering.target, resource_entity);
    assert!(gatherer.get::<IntendsToGather>().is_none());
}

#[test]
fn test_gathering_fails_on_mismatched_resource() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, gathering_system);

    // Create a tree resource with "wood"
    let tree_entity = app
        .world
        .spawn((
            ResourceComponent {
                name: "wood".to_string(),
                quantity: 10,
            },
            Position { x: 1, y: 1 },
        ))
        .id();

    // Create a gatherer that is adjacent to the tree
    let gatherer_entity = app
        .world
        .spawn((
            Inventory::new(),
            KnownResources(HashMap::new()),
            Position { x: 1, y: 2 },
            // But the intent is to gather "stone"!
            IsGathering {
                target: tree_entity,
                resource: "stone".to_string(),
                amount: 5,
            },
        ))
        .id();

    // 2. Run the system
    app.update();

    // 3. Verify
    // The gatherer's intent should be removed because it's invalid.
    assert!(
        app.world.get::<IsGathering>(gatherer_entity).is_none(),
        "IsGathering intent should be removed for mismatched resource"
    );

    // The gatherer should NOT have received any "stone".
    let gatherer_inventory = app.world.get::<Inventory>(gatherer_entity).unwrap();
    assert_eq!(
        gatherer_inventory.get_quantity("stone"),
        0,
        "Gatherer should have 0 stone"
    );

    // The tree's "wood" quantity should be unchanged.
    let tree_resource = app.world.get::<ResourceComponent>(tree_entity).unwrap();
    assert_eq!(
        tree_resource.quantity,
        10,
        "Tree wood quantity should not have changed"
    );
}
