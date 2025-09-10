use rust_simulation::{
    components::{
        intents::{CheckResources, HasResources, IntendsToBuild, TileIsSuitable},
        Inventory, Position,
    },
    map::{Map, Tile, CHUNK_SIZE},
    systems::building_logic::{check_resources_system, check_tile_system},
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

#[test]
fn test_check_tile_system_adjacent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let map = Map::new(10, 10, "data/biomes.json", "data/resources.json")
        .expect("Failed to create map");

    let build_pos = Position { x: 5, y: 5 };

    // Manually set the tile to be buildable
    {
        let (chunk_x, chunk_y) = map.get_chunk_index(build_pos.x, build_pos.y).unwrap();
        let mut chunk = map.chunks[chunk_y as usize][chunk_x as usize]
            .lock()
            .unwrap();
        let local_x = (build_pos.x % CHUNK_SIZE) as usize;
        let local_y = (build_pos.y % CHUNK_SIZE) as usize;
        chunk.tiles[local_y][local_x] = Tile::new('.', "grassland".to_string());
    }
    app.insert_resource(map);

    let entity = app
        .world
        .spawn((
            Position { x: 5, y: 4 }, // agent position
            IntendsToBuild {
                structure: "chest".to_string(),
                position: build_pos,
            },
            HasResources,
        ))
        .id();

    app.add_systems(Update, check_tile_system);
    app.update();

    assert!(app
        .world
        .entity(entity)
        .get::<TileIsSuitable>()
        .is_some());
}
