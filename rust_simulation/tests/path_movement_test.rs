use bevy::prelude::*;
use rust_simulation::{
    components::{
        path::{CurrentPath, PathRequest},
        Position, Velocity,
    },
    systems::path_movement_system::path_movement_system,
};
use std::collections::VecDeque;

const STUCK_THRESHOLD: u32 = 5; // Should match the planned implementation

#[test]
fn test_agent_becomes_unstuck_and_replans() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, path_movement_system);

    // Create an agent with a path
    let mut path = VecDeque::new();
    path.push_back((1, 2));
    path.push_back((1, 3));

    let agent_entity = app
        .world
        .spawn((
            Position { x: 1, y: 1 },
            CurrentPath {
                nodes: path,
                stuck_ticks: 0,
            },
        ))
        .id();

    // 2. Run system multiple times without updating position
    // This simulates the agent being blocked by an obstacle.

    // Run update once to allow the `Added<CurrentPath>` change detection to trigger.
    app.update();

    for _ in 0..=STUCK_THRESHOLD {
        // Run the system to update velocity and potentially the stuck counter
        app.update();

        // The agent should have a velocity component trying to move it
        assert!(app.world.get::<Velocity>(agent_entity).is_some());

        // In a real scenario, the movement system would see the Velocity,
        // try to move the agent, and then remove the Velocity component.
        // To simulate being stuck, we just don't update the Position,
        // and we let the Velocity component remain for the next tick.
        // The `path_movement_system` will see the leftover Velocity
        // and increment its stuck counter.
    }

    // 3. Verify
    // The agent should no longer have a path
    assert!(
        app.world.get::<CurrentPath>(agent_entity).is_none(),
        "CurrentPath should be removed after being stuck for too long"
    );

    // The agent should now have a PathRequest to try again
    assert!(
        app.world.get::<PathRequest>(agent_entity).is_some(),
        "A new PathRequest should be made after getting stuck"
    );
}
