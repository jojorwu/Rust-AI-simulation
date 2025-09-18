use bevy::prelude::*;
use rust_simulation::{
    components::{
        ai::KnownResources,
        intents::{IntendsToGather, IsGathering},
        path::PathRequest,
        Inventory, Position, Resource as ResourceComponent,
    },
    map::Map,
    systems::{find_resource::find_resource_system, gathering::gathering_system},
};
use std::collections::{HashMap, HashSet};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(
        Map::new(10, 10, "data/biomes.json", "data/resources.json")
            .expect("Failed to create map"),
    );
    app
}

#[test]
fn test_find_resource_system() {
    let mut app = setup_test_app();
    let resource_pos = Position { x: 5, y: 5 };
    let resource_entity = app
        .world
        .spawn((
            ResourceComponent {
                name: "wood".to_string(),
                quantity: 10,
            },
            resource_pos,
        ))
        .id();
    app.world
        .resource_mut::<Map>()
        .add_entity_to_spatial_map(resource_entity, 5, 5);

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
fn test_gathering_system_success() {
    let mut app = setup_test_app();
    let resource_pos = Position { x: 5, y: 5 };
    let resource_entity = app
        .world
        .spawn((
            ResourceComponent {
                name: "wood".to_string(),
                quantity: 10,
            },
            resource_pos,
        ))
        .id();
    app.world
        .resource_mut::<Map>()
        .add_entity_to_spatial_map(resource_entity, 5, 5);

    let gatherer_pos = Position { x: 4, y: 5 };
    let gatherer_entity = app
        .world
        .spawn((
            KnownResources(HashMap::new()),
            gatherer_pos,
            Inventory::new(),
            IsGathering {
                target: resource_entity,
                resource: "wood".to_string(),
                amount: 1,
            },
        ))
        .id();

    app.add_systems(Update, gathering_system);
    app.update();

    let gatherer = app.world.entity(gatherer_entity);
    assert!(gatherer.get::<IsGathering>().is_none());
    let inventory = gatherer.get::<Inventory>().unwrap();
    assert_eq!(inventory.get_quantity("wood"), 1);

    let resource = app.world.get::<ResourceComponent>(resource_entity).unwrap();
    assert_eq!(resource.quantity, 9);
}

#[test]
fn test_gathering_system_path_request() {
    let mut app = setup_test_app();
    let resource_pos = Position { x: 5, y: 5 };
    let resource_entity = app
        .world
        .spawn((
            ResourceComponent {
                name: "wood".to_string(),
                quantity: 10,
            },
            resource_pos,
        ))
        .id();
    app.world
        .resource_mut::<Map>()
        .add_entity_to_spatial_map(resource_entity, 5, 5);

    let gatherer_entity = app
        .world
        .spawn((
            KnownResources(HashMap::new()),
            Position { x: 0, y: 0 },
            Inventory::new(),
            IsGathering {
                target: resource_entity,
                resource: "wood".to_string(),
                amount: 1,
            },
        ))
        .id();

    app.add_systems(Update, gathering_system);
    app.update();

    let gatherer = app.world.entity(gatherer_entity);
    assert!(gatherer.get::<PathRequest>().is_some());
    assert!(gatherer.get::<IsGathering>().is_none());
}

#[test]
fn test_find_resource_system_removes_invalid_known_resource() {
    let mut app = setup_test_app();
    let mut known_resources = KnownResources(HashMap::new());
    let mut positions = HashSet::new();
    let invalid_pos = Position { x: 5, y: 5 };
    positions.insert(invalid_pos);
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

    let known_resources = app.world.get::<KnownResources>(gatherer_entity).unwrap();
    let wood_positions = known_resources.0.get("wood").unwrap();
    assert!(!wood_positions.contains(&invalid_pos));
}
