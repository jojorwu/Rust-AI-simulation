use bevy::prelude::*;
use rust_simulation::{
    components::{
        ai::MentalMap,
        path::{CurrentPath, PathRequest},
        Position,
    },
    player::Player,
    systems::{
        async_result_collection_system::async_result_collection_system,
        path_collection_system::path_collection_system,
        pathfinding_system::pathfinding_system,
    },
    async_task::AsyncResultChannel,
    brain::MemoryTile,
    map::Tile,
    config::{WIDTH, HEIGHT},
};

#[test]
fn test_pathfinding_flow() {
    // Create a new Bevy App
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);

    // --- Setup Test Data ---
    let mut mental_map = MentalMap(vec![vec![None; WIDTH as usize]; HEIGHT as usize]);
    // Create a walkable path
    for i in 0..5 {
        mental_map.0[0][i] = Some(MemoryTile {
            tile: Tile::new('.', "grassland".to_string()),
        });
    }
    // Create a wall
    mental_map.0[1][1] = Some(MemoryTile {
        tile: Tile::new('#', "wall".to_string()),
    });


    app.init_resource::<AsyncResultChannel>();

    // --- Setup Systems ---
    app.add_systems(
        Update,
        (
            pathfinding_system,
            async_result_collection_system,
            path_collection_system,
        ),
    );

    // --- Setup Entities ---
    let start_pos = (0, 0);
    let goal_pos = (4, 0);
    let player_entity = app
        .world
        .spawn((
            Player::new(0, WIDTH, HEIGHT),
            Position {
                x: start_pos.0,
                y: start_pos.1,
            },
            PathRequest {
                start: start_pos,
                goal: goal_pos,
            },
            mental_map, // Add the mental map component
        ))
        .id();

    // --- Run App ---
    for _ in 0..10 {
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // --- Check for Path ---
    let player_entity_ref = app.world.entity(player_entity);
    assert!(
        player_entity_ref.get::<CurrentPath>().is_some(),
        "Path was not found"
    );
}
