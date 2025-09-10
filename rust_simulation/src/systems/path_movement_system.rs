use crate::components::{Position, Velocity, path::CurrentPath};
use bevy_ecs::prelude::*;
use log::debug;

pub fn path_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CurrentPath, &Position), Or<(Changed<Position>, Added<CurrentPath>)>>,
) {
    for (entity, mut path, position) in query.iter_mut() {
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
