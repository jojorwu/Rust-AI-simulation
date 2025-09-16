use rust_simulation::{
    components::{intents::IntendsToBuild, Inventory, Position},
    events::Event,
    map::Map,
    systems::building::building_system,
    RecipeManagerResource, recipes::RecipeManager,
};
use bevy::prelude::*;
use std::sync::Arc;

#[test]
fn test_building_system_sends_build_request() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();

    let recipe_manager = Arc::new(
        RecipeManager::new("data/recipes.json").expect("Failed to create recipe manager"),
    );
    app.insert_resource(RecipeManagerResource(recipe_manager));

    let map = Map::new(10, 10, "data/biomes.json", "data/resources.json")
        .expect("Failed to create map");
    map.set_tile(5, 5, rust_simulation::map::Tile::new('.', "grassland".to_string()));
    app.insert_resource(map);

    let mut inventory = Inventory::new();
    inventory.add_item("wood", 25);
    inventory.add_item("stone", 10);

    let _builder_entity = app
        .world
        .spawn((
            Position { x: 5, y: 5 },
            inventory,
            IntendsToBuild("chest".to_string()),
        ))
        .id();

    app.add_systems(Update, building_system);
    app.update();

    let events = app.world.resource::<Events<Event>>();
    let mut reader = events.get_reader();
    let mut build_request_sent = false;
    for event in reader.read(events) {
        println!("Event received: {event:?}");
        if let Event::BuildRequest { .. } = event {
            build_request_sent = true;
            break;
        }
    }
    assert!(build_request_sent);
}

#[test]
fn test_agent_does_not_get_stuck_on_invalid_tile() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();

    let recipe_manager = Arc::new(
        rust_simulation::recipes::RecipeManager::new("data/recipes.json")
            .expect("Failed to create recipe manager"),
    );
    app.insert_resource(RecipeManagerResource(recipe_manager));

    let map = Map::new(10, 10, "data/biomes.json", "data/resources.json").unwrap();
    // Make the target tile non-buildable (e.g., water)
    map.set_tile(5, 5, rust_simulation::map::Tile::new('~', "water".to_string()));
    app.insert_resource(map);

    // Add the new, single building system
    app.add_systems(Update, building_system);

    // Create an agent with enough resources that wants to build on the invalid tile
    let mut inventory = Inventory::new();
    inventory.add_item("wood", 50);
    let agent_entity = app
        .world
        .spawn((
            inventory,
            Position { x: 5, y: 5 },
            IntendsToBuild("wall".to_string()),
        ))
        .id();

    // 2. Run systems for one tick
    app.update();

    // 3. Verify
    let agent = app.world.entity(agent_entity);
    // After the refactoring, the IntendsToBuild component should be removed even on failure.
    assert!(
        agent.get::<IntendsToBuild>().is_none(),
        "IntendsToBuild should be removed even on failure"
    );
}
