use crate::components::{Position, Velocity, path::CurrentPath};
use bevy_ecs::prelude::*;
use log::debug;

use crate::components::path::PathRequest;

const STUCK_THRESHOLD: u32 = 5;

type PathMovementQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut CurrentPath,
        &'static Position,
        Option<&'static Velocity>,
    ),
    Or<(Changed<Position>, Added<CurrentPath>, With<Velocity>)>,
>;

pub fn path_movement_system(mut commands: Commands, mut query: PathMovementQuery) {
    for (entity, mut path, position, velocity) in query.iter_mut() {
        if velocity.is_some() {
            // If entity has a velocity, it means it tried to move last tick but didn't.
            path.stuck_ticks += 1;
        } else {
            // No velocity means it's either just started or successfully moved.
            path.stuck_ticks = 0;
        }

        if path.stuck_ticks > STUCK_THRESHOLD {
            debug!("Entity {entity:?} is stuck, clearing path and replanning.");
            // Get the goal before we remove the path
            if let Some(goal_node) = path.nodes.back().cloned() {
                commands.entity(entity).insert(PathRequest {
                    start: (position.x, position.y),
                    goal: goal_node,
                });
            }
            commands.entity(entity).remove::<CurrentPath>();
            continue; // Move to the next entity
        }

        // If the agent is at the current head of the path, pop it.
        // This handles both the starting node and arriving at an intermediate node.
        if let Some(&next_node) = path.nodes.front() {
            if next_node.0 == position.x && next_node.1 == position.y {
                path.nodes.pop_front();
                debug!("Entity {entity:?} arrived at path node {next_node:?}");
            }
        }

        // After potentially popping the current node, if there's a next one, move towards it.
        if let Some(target_node) = path.nodes.front() {
            let dx = target_node.0 as i32 - position.x as i32;
            let dy = target_node.1 as i32 - position.y as i32;

            debug!(
                "Entity {entity:?} moving towards {target_node:?} with velocity ({dx}, {dy})"
            );
            commands.entity(entity).insert(Velocity { dx, dy });
        } else {
            // No nodes left, the path is complete.
            debug!("Entity {entity:?} finished its path.");
            commands.entity(entity).remove::<CurrentPath>();
        }
    }
}
