use criterion::{criterion_group, criterion_main, Criterion};
use rust_simulation::{
    components::{
        path::{CurrentPath, PathRequest},
    },
    map::{Map, Tile},
    player::Player,
    systems::{
        async_result_collection_system::async_result_collection_system,
        pathfinding_system::pathfinding_system,
    },
    DataPaths, setup_simulation
};
use bevy::prelude::*;
use std::env;

fn setup_app() -> App {
    let mut app = App::new();

    // Add minimal plugins for a headless run.
    app.add_plugins(TaskPoolPlugin::default());
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::log::LogPlugin::default());

    // Insert resources needed for setup
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    app.insert_resource(DataPaths {
        biomes: format!("{manifest_dir}/data/biomes.json"),
        resources: format!("{manifest_dir}/data/resources.json"),
        items: format!("{manifest_dir}/data/items.json"),
        recipes: format!("{manifest_dir}/data/recipes.json"),
    });

    // Add setup and pathfinding systems
    app.add_systems(Startup, setup_simulation);
    app.add_systems(Update, (pathfinding_system, async_result_collection_system).chain());

    // Run startup systems to create the world
    app.update();

    app
}

fn pathfinding_benchmark(c: &mut Criterion) {
    // Setup the app once
    let mut app = setup_app();

    // Get the player entity
    let player_entity = app.world.query_filtered::<Entity, With<Player>>().iter(&app.world).next().unwrap();

    // Set up the specific map condition for the test
    {
        let mut map = app.world.resource_mut::<Map>();
        map.set_tile(1, 0, Tile::new('.', "grassland".to_string()));
    }

    c.bench_function("pathfinding_flow", |b| {
        // b.iter runs the closure many times and measures its performance.
        b.iter(|| {
            // Add the request to the player
            app.world.entity_mut(player_entity).insert(PathRequest {
                start: (0, 0),
                goal: (1, 0),
            });

            // Run the app update loop until the path is found
            let mut path_found = false;
            for _ in 0..10 { // Max 10 updates
                app.update();
                if app.world.get::<CurrentPath>(player_entity).is_some() {
                    path_found = true;
                    break;
                }
            }
            assert!(path_found);

            // Clean up for the next iteration
            app.world.entity_mut(player_entity).remove::<CurrentPath>();
            app.world.entity_mut(player_entity).remove::<PathRequest>();
        });
    });
}

criterion_group!(benches, pathfinding_benchmark);
criterion_main!(benches);
