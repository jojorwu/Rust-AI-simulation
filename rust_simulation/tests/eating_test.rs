use bevy::prelude::*;
use rust_simulation::{
    components::{intents::WantsToEat, status::Hunger, Inventory},
    item::ItemRegistry,
    systems::eating::eating_system,
    ItemRegistryResource,
};
use std::sync::Arc;

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Setup item registry with a "berries" food item
    let mut items = std::collections::HashMap::new();
    items.insert(
        "berries".to_string(),
        rust_simulation::item::Item {
            name: "berries".to_string(),
            stackable: true,
            tool: false,
            is_food: true,
            food_value: 15.0,
            properties: None,
            category: None,
        },
    );
    let item_registry = ItemRegistry { items };
    app.insert_resource(ItemRegistryResource(Arc::new(item_registry)));

    app.add_systems(Update, eating_system);
    app
}

#[test]
fn test_eating_succeeds_with_food() {
    // 1. Setup
    let mut app = setup_test_app();

    let mut inventory = Inventory::new();
    inventory.add_item("berries", 1);

    let agent_entity = app
        .world
        .spawn((
            inventory,
            Hunger {
                current: 50.0,
                max: 100.0,
            },
            WantsToEat("berries".to_string()),
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let agent = app.world.entity(agent_entity);
    assert!(
        agent.get::<WantsToEat>().is_none(),
        "WantsToEat intent should be removed after successful eating"
    );

    let hunger = agent.get::<Hunger>().unwrap();
    assert_eq!(hunger.current, 65.0); // 50 + 15

    let inventory = agent.get::<Inventory>().unwrap();
    assert_eq!(inventory.get_quantity("berries"), 0);
}

#[test]
fn test_eating_fails_without_food_removes_intent() {
    // 1. Setup
    let mut app = setup_test_app();

    let agent_entity = app
        .world
        .spawn((
            Inventory::new(), // Empty inventory
            Hunger {
                current: 50.0,
                max: 100.0,
            },
            WantsToEat("berries".to_string()),
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let agent = app.world.entity(agent_entity);
    assert!(
        agent.get::<WantsToEat>().is_none(),
        "WantsToEat intent should be removed even if eating fails"
    );

    let hunger = agent.get::<Hunger>().unwrap();
    assert_eq!(
        hunger.current, 50.0,
        "Hunger should not change if eating fails"
    );
}
