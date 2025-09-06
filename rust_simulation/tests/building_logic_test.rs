use rust_simulation::{
    components::{
        intents::{CheckResources, HasResources},
        Inventory,
    },
    systems::building_logic::check_resources_system,
    RecipeManagerResource,
};
use bevy::prelude::*;
use std::sync::Arc;

#[test]
fn test_check_resources_system_success() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let recipe_manager = Arc::new(
        rust_simulation::recipes::RecipeManager::new("data/recipes.json")
            .expect("Failed to create recipe manager"),
    );
    app.insert_resource(RecipeManagerResource(recipe_manager));

    let mut inventory = Inventory::new();
    inventory.add_item("wood", 25);
    inventory.add_item("stone", 10);

    let entity = app
        .world
        .spawn((inventory, CheckResources("chest".to_string())))
        .id();

    app.add_systems(Update, check_resources_system);
    app.update();

    let entity_ref = app.world.entity(entity);
    assert!(entity_ref.get::<HasResources>().is_some());
    assert!(entity_ref.get::<CheckResources>().is_none());
}

#[test]
fn test_check_resources_system_insufficient_resources() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let recipe_manager = Arc::new(
        rust_simulation::recipes::RecipeManager::new("data/recipes.json")
            .expect("Failed to create recipe manager"),
    );
    app.insert_resource(RecipeManagerResource(recipe_manager));

    // Inventory with not enough wood
    let mut inventory = Inventory::new();
    inventory.add_item("wood", 5);
    inventory.add_item("stone", 10);

    let entity = app
        .world
        .spawn((inventory, CheckResources("chest".to_string())))
        .id();

    app.add_systems(Update, check_resources_system);
    app.update();

    let entity_ref = app.world.entity(entity);
    assert!(entity_ref.get::<HasResources>().is_none());
    assert!(entity_ref.get::<CheckResources>().is_none());
}
