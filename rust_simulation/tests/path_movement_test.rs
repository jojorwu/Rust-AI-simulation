use bevy::prelude::*;
use rust_simulation::{
    components::{
        path::{CurrentPath, PathRequest},
        Position, Velocity,
    },
    systems::path_movement_system::path_movement_system,
};
use std::collections::{HashSet, VecDeque};

// A simple movement system for the test that respects blockers.
fn simple_movement_system(
    mut query: Query<(&mut Position, &Velocity)>,
    blocker_query: Query<&Position, Without<Velocity>>,
) {
    let blockers: HashSet<_> = blocker_query.iter().map(|p| (p.x, p.y)).collect();
    for (mut pos, vel) in query.iter_mut() {
        let new_pos = (pos.x as i32 + vel.dx, pos.y as i32 + vel.dy);
        if !blockers.contains(&(new_pos.0 as u32, new_pos.1 as u32)) {
            pos.x = new_pos.0 as u32;
            pos.y = new_pos.1 as u32;
        }
    }
}

#[test]
fn test_agent_gets_stuck_and_repaths() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (path_movement_system, simple_movement_system).chain());

    // Create an agent with a path
    let mut path = VecDeque::new();
    path.push_back((0, 0));
    path.push_back((0, 1));
    path.push_back((0, 2));
    let agent_entity = app
        .world
        .spawn((
            Position { x: 0, y: 0 },
            CurrentPath {
                nodes: path,
                stuck_timer: 0,
            },
            // Velocity will be added by the path_movement_system
        ))
        .id();

    // Create a blocker on the path
    app.world.spawn(Position { x: 0, y: 1 });

    // 2. Run the systems for enough ticks to get stuck
    // The agent should move to (0,0) on tick 1 (and pop it), then try to move to (0,1)
    // It should fail to move for ticks 2, 3, 4, 5, 6.
    // On tick 7, it should give up and request a new path.
    for _ in 0..10 {
        app.update();
    }

    // 3. Verify
    // The agent should no longer have a CurrentPath.
    assert!(
        app.world.entity(agent_entity).get::<CurrentPath>().is_none(),
        "Agent should have given up on its path"
    );
    // The agent should now have a PathRequest to try again.
    assert!(
        app.world.entity(agent_entity).get::<PathRequest>().is_some(),
        "Agent should have requested a new path"
    );
}
