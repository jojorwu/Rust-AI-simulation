use bevy::prelude::*;
use rust_simulation::{
    components::{Inventory, Position, WantsToPickup},
    map::Map,
    systems::pickup::pickup_system,
};

#[test]
fn test_pickup_intent_is_not_removed_if_no_item() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Map::new(10, 10, "data/biomes.json", "data/resources.json").unwrap());
    app.add_systems(Update, pickup_system);

    let picker_entity = app
        .world
        .spawn((
            Position { x: 5, y: 5 },
            Inventory::new(),
            WantsToPickup {},
        ))
        .id();

    app.update();

    assert!(app
        .world
        .get::<WantsToPickup>(picker_entity)
        .is_some());
}
