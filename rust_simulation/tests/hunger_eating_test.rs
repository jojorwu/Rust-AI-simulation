use bevy::prelude::*;
use rust_simulation::{
    ItemRegistryResource,
    components::{
        intents::WantsToEat,
        status::{Health, Hunger},
        Inventory,
    },
    config::Config,
    systems::{eating::eating_system, hunger::hunger_system},
};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Load a default config for the tests
    let config = Config::load("data/config.toml").expect("Failed to load config");
    let item_registry =
        rust_simulation::item::ItemRegistry::new("data/items.json").unwrap();
    app.insert_resource(config);
    app.insert_resource(ItemRegistryResource(std::sync::Arc::new(
        item_registry,
    )));
    app.add_systems(Update, (hunger_system, eating_system).chain());
    app
}

#[test]
fn test_hunger_and_starvation() {
    // 1. Setup
    let mut app = setup_test_app();
    let entity = app
        .world
        .spawn((
            Hunger { current: 1.0, max: 100.0 },
            Health { current: 100, max: 100 },
        ))
        .id();

    // 2. Run system enough times to cause starvation
    // HUNGER_RATE is 0.01, so 100 ticks to reach 0 hunger.
    // STARVATION_DAMAGE is 1.
    for _ in 0..101 {
        app.update();
    }

    // 3. Verify
    let hunger = app
        .world
        .get::<Hunger>(entity)
        .expect("Entity should have a Hunger component");
    let health = app
        .world
        .get::<Health>(entity)
        .expect("Entity should have a Health component");
    assert_eq!(hunger.current, 0.0);
    assert_eq!(health.current, 99); // 1 tick of starvation damage
}

#[test]
fn test_eating_restores_hunger() {
    // 1. Setup
    let mut app = setup_test_app();
    let mut inventory = Inventory::new();
    inventory.add_item("meat", 1);

    let initial_hunger = 50.0;
    let entity = app
        .world
        .spawn((
            Hunger { current: initial_hunger, max: 100.0 },
            inventory,
            WantsToEat("meat".to_string()),
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let config = app.world.resource::<Config>();
    let hunger = app
        .world
        .get::<Hunger>(entity)
        .expect("Entity should have a Hunger component");
    let inventory = app
        .world
        .get::<Inventory>(entity)
        .expect("Entity should have an Inventory component");

    let expected_hunger = initial_hunger - config.survival.hunger_rate + config.survival.meat_hunger_value;
    assert!((hunger.current - expected_hunger).abs() < 1e-6, "Expected hunger to be close to {}, but it was {}", expected_hunger, hunger.current);
    assert_eq!(inventory.get_quantity("meat"), 0);
    assert!(app.world.get::<WantsToEat>(entity).is_none());
}

#[test]
fn test_eating_does_not_exceed_max_hunger() {
    // 1. Setup
    let mut app = setup_test_app();
    let mut inventory = Inventory::new();
    inventory.add_item("meat", 1);

    let entity = app
        .world
        .spawn((
            Hunger { current: 90.0, max: 100.0 },
            inventory,
            WantsToEat("meat".to_string()),
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let hunger = app
        .world
        .get::<Hunger>(entity)
        .expect("Entity should have a Hunger component");
    // Should be clamped to max
    assert_eq!(hunger.current, 100.0);
}

#[test]
fn test_eating_removes_intent_when_no_food_present() {
    // 1. Setup
    let mut app = setup_test_app();
    let inventory = Inventory::new(); // Empty inventory

    let entity = app
        .world
        .spawn((
            Hunger { current: 50.0, max: 100.0 },
            inventory,
            WantsToEat("meat".to_string()),
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    // The intent should be removed even if eating fails, to prevent getting stuck.
    assert!(app.world.get::<WantsToEat>(entity).is_none());
}
