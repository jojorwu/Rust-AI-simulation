use rust_simulation::{
    components::{
        intents::{IntendsToGather, IsGathering},
        Position,
    },
    map::Map,
    spatial::SpatialIndex,
    systems::{find_resource::find_resource_system, spatial_indexing::update_spatial_index_system},
};
use bevy::prelude::*;

#[test]
fn test_find_resource_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SpatialIndex>();

    let mut map = Map::new(10, 10, "data/biomes.json", "data/resources.json")
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
    map.add_entity_to_spatial_map(resource_entity, 5, 5)
        .unwrap();
    app.insert_resource(map);

    let gatherer_entity = app
        .world
        .spawn((
            Position { x: 0, y: 0 },
            IntendsToGather("wood".to_string(), 1),
        ))
        .id();

    app.add_systems(Update, (update_spatial_index_system, find_resource_system).chain());
    app.update();

    let gatherer = app.world.entity(gatherer_entity);
    let is_gathering = gatherer
        .get::<IsGathering>()
        .expect("Gatherer should have IsGathering component");
    assert_eq!(is_gathering.target, resource_entity);
    assert!(gatherer.get::<IntendsToGather>().is_none());
}
