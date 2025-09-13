use bevy::prelude::*;
use rust_simulation::{
    components::{intents::WantsToEat, status::Hunger, Inventory},
    systems::eating::eating_system,
    ItemRegistryResource,
};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let item_registry =
        rust_simulation::item::ItemRegistry::new("data/items.json").unwrap();
    app.insert_resource(ItemRegistryResource(std::sync::Arc::new(
        item_registry,
    )));
    app.add_systems(Update, eating_system);
    app
}

#[test]
fn test_eating_fails_if_no_food() {
    // 1. Setup
    let mut app = setup_test_app();
    let entity = app
        .world
        .spawn((
            Hunger {
                current: 50.0,
                max: 100.0,
            },
            Inventory::new(),
            WantsToEat("meat".to_string()),
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let wants_to_eat = app.world.get::<WantsToEat>(entity);
    assert!(
        wants_to_eat.is_none(),
        "WantsToEat component should be removed even if eating fails"
    );
}
