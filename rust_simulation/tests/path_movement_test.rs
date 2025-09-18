use bevy::prelude::*;
use rust_simulation::{
    components::{path::CurrentPath, Position, Velocity},
    systems::path_movement_system::path_movement_system,
};
use std::collections::VecDeque;

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, path_movement_system);
    app
}

#[test]
fn test_path_movement_clamps_velocity() {
    // 1. Setup
    let mut app = setup_test_app();

    let mut path = CurrentPath {
        nodes: VecDeque::new(),
    };
    path.nodes.push_back((0, 0));
    path.nodes.push_back((2, 2)); // A gap in the path

    let entity = app.world.spawn((Position { x: 0, y: 0 }, path)).id();

    // 2. Run system
    app.update();

    // 3. Verify
    let velocity = app
        .world
        .get::<Velocity>(entity)
        .expect("Entity should have a Velocity component");
    assert_eq!(velocity.dx, 1, "dx should be clamped to 1");
    assert_eq!(velocity.dy, 1, "dy should be clamped to 1");
}
