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
fn test_check_resources_system() {
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

    assert!(app.world.entity(entity).get::<HasResources>().is_some());
}

use rust_simulation::{
    components::Position,
    map::{Map, Tile},
    systems::building_logic::check_tile_system,
};

#[test]
fn test_check_tile_system_fails_on_unsuitable_tile() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let map = Map::new(10, 10, "data/biomes.json", "data/resources.json")
        .expect("Failed to create map");
    app.insert_resource(map);
    app.add_systems(Update, check_tile_system);

    // Create an entity on a water tile, which is not suitable for building
    let position = Position { x: 0, y: 0 };
    let entity = app
        .world
        .spawn((
            position,
            HasResources,
            CheckResources("chest".to_string()),
        ))
        .id();

    // Manually set the tile to be unsuitable
    let mut map = app.world.resource_mut::<Map>();
    let (chunk_x, chunk_y) = map.get_chunk_index(position.x, position.y).unwrap();
    let mut chunk = map.chunks[chunk_y][chunk_x].try_lock().unwrap();
    chunk.tiles[0][0] = Tile::new('W', "ocean".to_string()); // Water tile
    drop(chunk); // Release the lock

    app.update();

    // Verify that the temporary components were removed
    let entity_ref = app.world.entity(entity);
    assert!(entity_ref.get::<HasResources>().is_none());
    assert!(entity_ref.get::<CheckResources>().is_none());
    assert!(entity_ref.get::<rust_simulation::components::intents::TileIsSuitable>().is_none());
}
