use rust_simulation::{
    components::{intents::IntendsToBuild, Inventory, Position},
    events::Event,
    map::Map,
    systems::{
        building::building_system,
        building_logic::{build_system, check_resources_system, check_tile_system},
    },
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

    app.add_systems(
        Update,
        (
            building_system,
            check_resources_system,
            check_tile_system,
            build_system,
        )
            .chain(),
    );
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
