use bevy::prelude::*;
use rust_simulation::{
    animals::pig::Pig,
    components::{DroppedItem, Position},
    events::Event,
    map::Map,
    systems::death::death_system,
};

// Helper to setup a basic app for testing death-related systems
fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();
    app.insert_resource(
        Map::new(10, 10, "data/biomes.json", "data/resources.json")
            .expect("Failed to create map"),
    );
    app.add_systems(Update, death_system);
    app
}

#[test]
fn test_death_system_despawns_entity() {
    // 1. Setup
    let mut app = setup_test_app();

    // Create a generic entity and add it to the map
    let entity_pos = Position { x: 5, y: 5 };
    let dead_entity = app.world.spawn(entity_pos).id();
    app.world
        .resource_mut::<Map>()
        .add_entity_to_spatial_map(dead_entity, entity_pos.x, entity_pos.y);

    // Verify entity exists before death
    assert!(app.world.get_entity(dead_entity).is_some());
    assert_eq!(
        app.world
            .resource::<Map>()
            .get_entities_at(entity_pos.x, entity_pos.y)
            .expect("Entities should be present")
            .len(),
        1
    );

    // 2. Send death event
    app.world.send_event(Event::EntityDied(dead_entity));
    app.update();

    // 3. Verify
    // Entity should be despawned
    assert!(app.world.get_entity(dead_entity).is_none());

    // Entity should be removed from spatial map
    let map_after_update = app.world.resource::<Map>();
    let entities_after = map_after_update.get_entities_at(entity_pos.x, entity_pos.y);
    assert!(entities_after.is_none_or(|e| e.is_empty()));
}

#[test]
fn test_death_system_pig_drops_meat() {
    // 1. Setup
    let mut app = setup_test_app();

    // Create a pig entity and add it to the map
    let pig_pos = Position { x: 3, y: 3 };
    let pig_entity = app.world.spawn((pig_pos, Pig {})).id();
    app.world
        .resource_mut::<Map>()
        .add_entity_to_spatial_map(pig_entity, pig_pos.x, pig_pos.y);

    // 2. Send death event
    app.world.send_event(Event::EntityDied(pig_entity));
    app.update();

    // 3. Verify
    // Pig entity should be despawned
    assert!(app.world.get_entity(pig_entity).is_none());

    // A "meat" item should be dropped at the pig's location
    let map_after_update = app.world.resource::<Map>();
    let entities_at_pos = map_after_update
        .get_entities_at(pig_pos.x, pig_pos.y)
        .expect("Entities should be present");
    assert_eq!(entities_at_pos.len(), 1);

    let dropped_item_entity = entities_at_pos[0];
    let item = app
        .world
        .get::<DroppedItem>(dropped_item_entity)
        .expect("DroppedItem component should be present");
    assert_eq!(item.item_name, "meat");
    assert_eq!(item.quantity, 1);
}
