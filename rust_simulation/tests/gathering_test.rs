use rust_simulation::{
    components::{
        ai::KnownResources,
        intents::{IntendsToGather, IntendsToGatherFrom},
        Position,
    },
    map::Map,
    systems::find_resource::find_resource_system,
};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[test]
fn test_find_resource_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let map = Map::new(10, 10, "data/biomes.json", "data/resources.json").unwrap();
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
            IntendsToGather("wood".to_string()),
        ))
        .id();

    app.add_systems(Update, find_resource_system);
    app.update();

    let gatherer = app.world.entity(gatherer_entity);
    assert!(gatherer.get::<IntendsToGatherFrom>().is_some());
    assert_eq!(
        gatherer.get::<IntendsToGatherFrom>().unwrap().0,
        resource_entity
    );
    assert!(gatherer.get::<IntendsToGather>().is_none());
}
