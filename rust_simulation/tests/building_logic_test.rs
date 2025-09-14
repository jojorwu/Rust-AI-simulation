use bevy::prelude::*;
use rust_simulation::{
    components::{
        intents::{CheckResources, HasResources, TileIsSuitable},
        Inventory, Position,
    },
    events::Event,
    map::Map,
    systems::building_logic::{build_system, check_resources_system, check_tile_system},
    RecipeManagerResource,
};
use std::sync::Arc;

#[test]
fn test_check_resources_system() {
    // Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let recipe_manager = Arc::new(
        rust_simulation::recipes::RecipeManager::new("data/recipes.json")
            .expect("Failed to create recipe manager"),
    );
    app.insert_resource(RecipeManagerResource(recipe_manager));

    // Create an entity with enough resources
    let mut inventory = Inventory::new();
    inventory.add_item("wood", 25);
    inventory.add_item("stone", 10);
    let entity = app
        .world
        .spawn((inventory, CheckResources("chest".to_string())))
        .id();

    // Run system
    app.add_systems(Update, check_resources_system);
    app.update();

    // Verify
    assert!(app.world.entity(entity).get::<HasResources>().is_some());
}

#[test]
fn test_build_system_cleans_up_on_failure() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();

    // Add resources
    let recipe_manager = Arc::new(
        rust_simulation::recipes::RecipeManager::new("data/recipes.json")
            .expect("Failed to create recipe manager"),
    );
    app.insert_resource(RecipeManagerResource(recipe_manager));
    let map = Map::new(1, 1, "data/biomes.json", "data/resources.json")
        .expect("Failed to create map");
    app.insert_resource(map); // Add a map so check_tile_system can run

    // Manually set the tile to be buildable, otherwise check_tile_system will fail
    let mut map = app.world.resource_mut::<Map>();
    let buildable_tile = rust_simulation::map::Tile::new('.', "plains".to_string());
    map.set_tile(0, 0, buildable_tile);

    // Add only the checking systems first
    app.add_systems(Update, (check_resources_system, check_tile_system).chain());

    // Create an entity with enough resources to pass the first check
    let mut inventory = Inventory::new();
    inventory.add_item("wood", 10);
    let agent_pos = Position { x: 0, y: 0 };
    let entity = app
        .world
        .spawn((
            inventory,
            agent_pos,
            CheckResources("door".to_string()),
        ))
        .id();

    // 2. Run initial checks (which should pass)
    app.update();

    // Verify that the agent is ready to build
    assert!(app.world.entity(entity).get::<HasResources>().is_some());
    assert!(app.world.entity(entity).get::<TileIsSuitable>().is_some());

    // 3. Manually remove the resources, simulating another system's action
    let mut agent_inventory = app.world.get_mut::<Inventory>(entity).unwrap();
    agent_inventory.remove_item("wood", 10);
    assert_eq!(agent_inventory.get_quantity("wood"), 0);

    // 4. Now, add the build system and run it. It should fail to get resources.
    app.add_systems(Update, build_system);
    app.update();

    // 5. Verify that the agent is NOT stuck
    let agent_entity = app.world.entity(entity);
    assert!(
        agent_entity.get::<CheckResources>().is_none(),
        "CheckResources should have been removed"
    );
    assert!(
        agent_entity.get::<HasResources>().is_none(),
        "HasResources should have been removed"
    );
    assert!(
        agent_entity.get::<TileIsSuitable>().is_none(),
        "TileIsSuitable should have been removed"
    );
}
