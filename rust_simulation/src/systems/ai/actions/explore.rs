use bevy_ecs::prelude::*;
use crate::components::{
    intents::IntendsToExplore, path::{CurrentPath, PathRequest}, BrainComponent, Position
};

pub fn explore_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &Position), (With<IntendsToExplore>, Without<CurrentPath>, Without<PathRequest>)>,
) {
    for (entity, mut brain, position) in query.iter_mut() {
        // Get the next destination from the frontier
        if let Some(target_pos) = brain.exploration_frontier.pop_front() {
            // If the target has become visible since being added to the frontier, skip it.
            if brain.mental_map[target_pos.y as usize][target_pos.x as usize].is_some() {
                // Try the next one on the next tick.
                continue;
            }

            // Request a path to the new frontier destination.
            commands.entity(entity).insert(PathRequest {
                start: (position.x, position.y),
                goal: (target_pos.x, target_pos.y),
            });
        } else {
            // No more frontiers to explore. The goal is complete.
            commands.entity(entity).remove::<IntendsToExplore>();
            brain.current_goal = None;
        }
    }
}
