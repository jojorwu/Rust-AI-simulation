use bevy::prelude::*;
use rust_simulation::{
    components::{intents::WantsToStoreItem, Chest, Inventory},
    systems::storage::storage_system,
};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, storage_system);
    app
}

#[test]
fn test_storage_system_success() {
    // 1. Setup
    let mut app = setup_test_app();

    let mut storer_inv = Inventory::new();
    storer_inv.add_item("wood", 10);

    let chest_entity = app.world.spawn(Chest { inventory: Inventory::new() }).id();
    let storer_entity = app
        .world
        .spawn((
            storer_inv,
            WantsToStoreItem {
                item_name: "wood".to_string(),
                quantity: 5,
                target_chest: chest_entity,
            },
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let storer_inv = app
        .world
        .get::<Inventory>(storer_entity)
        .expect("Storer should have an Inventory component");
    assert_eq!(storer_inv.get_quantity("wood"), 5);

    let chest_inv = &app
        .world
        .get::<Chest>(chest_entity)
        .expect("Chest should have a Chest component")
        .inventory;
    assert_eq!(chest_inv.get_quantity("wood"), 5);
}

#[test]
fn test_storage_fails_if_item_not_present() {
    // 1. Setup
    let mut app = setup_test_app();

    let storer_inv = Inventory::new(); // Empty inventory
    let chest_entity = app.world.spawn(Chest { inventory: Inventory::new() }).id();
    let storer_entity = app
        .world
        .spawn((
            storer_inv,
            WantsToStoreItem {
                item_name: "wood".to_string(),
                quantity: 5,
                target_chest: chest_entity,
            },
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let storer_inv = app
        .world
        .get::<Inventory>(storer_entity)
        .expect("Storer should have an Inventory component");
    assert_eq!(storer_inv.get_quantity("wood"), 0);

    let chest_inv = &app
        .world
        .get::<Chest>(chest_entity)
        .expect("Chest should have a Chest component")
        .inventory;
    assert_eq!(chest_inv.get_quantity("wood"), 0);
}

#[test]
fn test_storage_fails_if_chest_does_not_exist() {
    // 1. Setup
    let mut app = setup_test_app();

    let mut storer_inv = Inventory::new();
    storer_inv.add_item("wood", 10);

    let invalid_chest_entity = Entity::from_raw(999); // An entity that doesn't exist
    let storer_entity = app
        .world
        .spawn((
            storer_inv,
            WantsToStoreItem {
                item_name: "wood".to_string(),
                quantity: 5,
                target_chest: invalid_chest_entity,
            },
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    // This assertion will fail initially, as the item is destroyed.
    // After the fix, the item should remain in the storer's inventory.
    let storer_inv = app
        .world
        .get::<Inventory>(storer_entity)
        .expect("Storer should have an Inventory component");
    assert_eq!(storer_inv.get_quantity("wood"), 10);

    // Also assert that the intent to store is still present
    assert!(app.world.get::<WantsToStoreItem>(storer_entity).is_some());
}
