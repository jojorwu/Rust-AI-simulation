use bevy::prelude::*;
use rust_simulation::{
    components::{Chest, Inventory, Position, WantsToStoreItem},
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

    // Make the entities adjacent
    let chest_entity = app
        .world
        .spawn((
            Chest {
                inventory: Inventory::new(),
            },
            Position { x: 0, y: 0 },
        ))
        .id();
    let storer_entity = app
        .world
        .spawn((
            storer_inv,
            Position { x: 0, y: 1 },
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
    let chest_entity = app
        .world
        .spawn((
            Chest {
                inventory: Inventory::new(),
            },
            Position { x: 0, y: 0 },
        ))
        .id();
    let storer_entity = app
        .world
        .spawn((
            storer_inv,
            Position { x: 0, y: 1 },
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
            Position { x: 0, y: 0 },
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
    let storer_inv = app
        .world
        .get::<Inventory>(storer_entity)
        .expect("Storer should have an Inventory component");
    assert_eq!(storer_inv.get_quantity("wood"), 10);
}

#[test]
fn test_storage_fails_if_not_adjacent() {
    // 1. Setup
    let mut app = setup_test_app();

    // Create a chest entity at (0,0)
    let chest_entity = app
        .world
        .spawn((
            Chest {
                inventory: Inventory::new(),
            },
            Position { x: 0, y: 0 },
        ))
        .id();

    // Create an agent at (10, 10) with 1 wood
    let mut agent_inventory = Inventory::new();
    agent_inventory.add_item("wood", 1);
    let agent_entity = app
        .world
        .spawn((
            agent_inventory,
            Position { x: 10, y: 10 },
            WantsToStoreItem {
                item_name: "wood".to_string(),
                quantity: 1,
                target_chest: chest_entity,
            },
        ))
        .id();

    // 2. Run the system
    app.update();

    // 3. Verify
    // The agent should still have its wood because it's too far away.
    let final_agent_inventory = app.world.get::<Inventory>(agent_entity).unwrap();
    assert_eq!(final_agent_inventory.get_quantity("wood"), 1);

    // The chest should still be empty.
    let final_chest_inventory = &app.world.get::<Chest>(chest_entity).unwrap().inventory;
    assert_eq!(final_chest_inventory.get_quantity("wood"), 0);
}

#[test]
fn test_storage_succeeds_if_adjacent() {
    // 1. Setup
    let mut app = setup_test_app();

    // Create a chest entity at (0,0)
    let chest_entity = app
        .world
        .spawn((
            Chest {
                inventory: Inventory::new(),
            },
            Position { x: 0, y: 0 },
        ))
        .id();

    // Create an agent at (0, 1) (adjacent) with 1 wood
    let mut agent_inventory = Inventory::new();
    agent_inventory.add_item("wood", 1);
    let agent_entity = app
        .world
        .spawn((
            agent_inventory,
            Position { x: 0, y: 1 },
            WantsToStoreItem {
                item_name: "wood".to_string(),
                quantity: 1,
                target_chest: chest_entity,
            },
        ))
        .id();

    // 2. Run the system
    app.update();

    // 3. Verify
    // The agent should no longer have its wood.
    let final_agent_inventory = app.world.get::<Inventory>(agent_entity).unwrap();
    assert_eq!(final_agent_inventory.get_quantity("wood"), 0);

    // The chest should now have the wood.
    let final_chest_inventory = &app.world.get::<Chest>(chest_entity).unwrap().inventory;
    assert_eq!(final_chest_inventory.get_quantity("wood"), 1);
}
