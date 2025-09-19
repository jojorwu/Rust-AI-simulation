use bevy::prelude::*;
use rust_simulation::{
    components::{intents::WantsToCraft, Inventory},
    recipes::RecipeManager,
    systems::crafting::crafting_system,
    RecipeManagerResource,
};
use std::sync::Arc;

// Helper to setup a basic app for testing
fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let recipe_manager = Arc::new(
        RecipeManager::new("data/recipes.json").expect("Failed to create recipe manager"),
    );
    app.insert_resource(RecipeManagerResource(recipe_manager));

    app.add_systems(Update, crafting_system);
    app
}

#[test]
fn test_crafting_system_success() {
    // 1. Setup
    let mut app = setup_test_app();

    // Create crafter with sufficient resources for a "stone_axe" (2 wood, 3 stone)
    let mut inventory = Inventory::new();
    inventory.add_item("wood", 2);
    inventory.add_item("stone", 3);
    let crafter_entity = app
        .world
        .spawn((inventory, WantsToCraft { item_name: "stone_axe".to_string() }))
        .id();

    // 2. Run the system
    app.update();

    // 3. Verify
    let inventory = app
        .world
        .get::<Inventory>(crafter_entity)
        .expect("Crafter should have an Inventory component");
    // Resources should be consumed
    assert_eq!(inventory.get_quantity("wood"), 0);
    assert_eq!(inventory.get_quantity("stone"), 0);
    // New item should be added
    assert_eq!(inventory.get_quantity("stone_axe"), 1);
    // Intent should be removed
    assert!(app.world.get::<WantsToCraft>(crafter_entity).is_none());
}

#[test]
fn test_crafting_system_insufficient_resources() {
    // 1. Setup
    let mut app = setup_test_app();

    // Create crafter with insufficient resources
    let mut inventory = Inventory::new();
    inventory.add_item("wood", 1); // Not enough for a stone_axe
    let crafter_entity = app
        .world
        .spawn((inventory, WantsToCraft { item_name: "stone_axe".to_string() }))
        .id();

    // 2. Run the system
    app.update();

    // 3. Verify
    let inventory = app
        .world
        .get::<Inventory>(crafter_entity)
        .expect("Crafter should have an Inventory component");
    // Resources should NOT be consumed
    assert_eq!(inventory.get_quantity("wood"), 1);
    // New item should NOT be added
    assert_eq!(inventory.get_quantity("stone_axe"), 0);
    // Intent should still be removed
    assert!(app.world.get::<WantsToCraft>(crafter_entity).is_none());
}

#[test]
fn test_crafting_circular_dependency() {
    // 1. Setup
    use std::collections::HashMap;
    let mut recipes = HashMap::new();
    // Create a circular dependency: item_a needs item_b, and item_b needs item_a
    let mut recipe_a = HashMap::new();
    recipe_a.insert("item_b".to_string(), 1);
    recipes.insert("item_a".to_string(), recipe_a);

    let mut recipe_b = HashMap::new();
    recipe_b.insert("item_a".to_string(), 1);
    recipes.insert("item_b".to_string(), recipe_b);

    let recipe_manager = RecipeManager::with_recipes(recipes);

    // 2. Run and Verify
    let result = recipe_manager.get_required_resources("item_a", 1);
    assert!(result.is_err(), "Should have detected a circular dependency");

    // We can also check the specific error type if we want to be more precise
    if let Err(e) = result {
        assert!(matches!(e, rust_simulation::errors::SimulationError::CircularDependency(_)));
    }
}
