use bevy::prelude::*;
use rust_simulation::{
    components::{intents::WantsToPickup, DroppedItem, Inventory, Position},
    map::Map,
    systems::pickup::pickup_system,
};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(
        Map::new(10, 10, "data/biomes.json", "data/resources.json")
            .expect("Failed to create map"),
    );
    app.add_systems(Update, pickup_system);
    app
}

#[test]
fn test_pickup_intent_remains_if_no_item() {
    // 1. Setup
    let mut app = setup_test_app();

    // Create a picker entity on a tile with no items
    let picker_pos = Position { x: 1, y: 1 };
    let picker_entity = app
        .world
        .spawn((picker_pos, Inventory::new(), WantsToPickup {}))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    // The WantsToPickup intent should NOT be removed
    assert!(
        app.world.get::<WantsToPickup>(picker_entity).is_some(),
        "WantsToPickup intent should remain if no item was picked up"
    );
}
