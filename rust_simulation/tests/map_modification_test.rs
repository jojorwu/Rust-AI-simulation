use bevy::prelude::*;
use rust_simulation::{
    components::{Chest, Position},
    events::Event,
    map::Map,
    systems::map_modification::map_modification_system,
};

#[test]
fn test_map_modification_builds_chest() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();
    app.insert_resource(Map::new(10, 10, "data/biomes.json", "data/resources.json").unwrap());
    app.add_systems(Update, map_modification_system);

    // Create a builder entity (doesn't need any components for this test)
    let builder_entity = app.world.spawn_empty().id();
    let build_pos = Position { x: 5, y: 5 };

    // Verify initial state
    let map = app.world.resource::<Map>();
    assert_ne!(map.get_tile(build_pos.x, build_pos.y).unwrap().tile_type, 'C');

    // 2. Send build request event
    app.world.send_event(Event::BuildRequest {
        builder: builder_entity,
        structure: "chest".to_string(),
        position: build_pos,
    });
    app.update();

    // 3. Verify final state
    let map_after_update = app.world.resource::<Map>();

    // Tile should be changed to 'C'
    assert_eq!(map_after_update.get_tile(build_pos.x, build_pos.y).unwrap().tile_type, 'C');

    // A chest entity should exist at the position
    let entities_at_pos = map_after_update.get_entities_at(build_pos.x, build_pos.y).unwrap();
    assert_eq!(entities_at_pos.len(), 1);

    // The entity should have a Chest component
    let chest_entity = entities_at_pos[0];
    assert!(app.world.get::<Chest>(chest_entity).is_some());
}
