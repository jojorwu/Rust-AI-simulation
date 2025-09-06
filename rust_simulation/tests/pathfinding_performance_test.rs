use bevy::prelude::*;
use rust_simulation::{
    brain::MemoryTile,
    components::{
        ai::MentalMap,
        path::{CurrentPath, PathRequest},
        Position,
    },
    map::Tile,
    systems::{
        pathfinding_completion_system::pathfinding_completion_system,
        pathfinding_system::pathfinding_system,
    },
};

#[test]
fn test_pathfinding_performance_long_path() {
    // This test is designed to be very slow or time out with the old, inefficient
    // pathfinding algorithm. With the refactored algorithm, it should pass quickly.

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    const MAP_SIZE: usize = 100;
    let mut mental_map = MentalMap(vec![vec![None; MAP_SIZE]; MAP_SIZE]);
    // Create a clear map
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            mental_map.0[y][x] = Some(MemoryTile {
                tile: Tile::new('.', "grassland".to_string()),
            });
        }
    }

    app.add_systems(
        Update,
        (
            pathfinding_system,
            pathfinding_completion_system,
        ),
    );

    let start_pos = (0, 0);
    let goal_pos = (MAP_SIZE as u32 - 1, MAP_SIZE as u32 - 1);
    let entity = app
        .world
        .spawn((
            Position { x: start_pos.0, y: start_pos.1 },
            PathRequest { start: start_pos, goal: goal_pos },
            mental_map,
        ))
        .id();

    // The number of updates might need to be high for the async task to complete.
    for _ in 0..20 {
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let entity_ref = app.world.entity(entity);
    assert!(
        entity_ref.get::<CurrentPath>().is_some(),
        "Path was not found for a long path. This might be a timeout or a bug."
    );
}
