use bevy::prelude::*;
use rust_simulation::{
    components::{intents::WantsToPickup, DroppedItem, Inventory, Position},
    map::Map,
    systems::pickup::pickup_system,
};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Map::new(10, 10, "data/biomes.json", "data/resources.json").unwrap());
    app.add_systems(Update, pickup_system);
    app
}

#[test]
fn test_pickup_intent_persists_if_item_is_gone() {
    // 1. Setup
    let mut app = setup_test_app();

    // Agent 1 wants to pick up an item that doesn't exist.
    let agent1_entity = app
        .world
        .spawn((
            Inventory::new(),
            Position { x: 1, y: 1 },
            WantsToPickup {},
        ))
        .id();

    // 2. Run the system
    app.update();

    // 3. Verify
    // The agent should still want to pick up an item, because it failed to do so.
    assert!(app.world.get::<WantsToPickup>(agent1_entity).is_some());
}
