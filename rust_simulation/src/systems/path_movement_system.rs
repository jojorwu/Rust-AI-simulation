use crate::components::{Position, Velocity, path::CurrentPath};
use bevy_ecs::prelude::*;
use log::debug;

pub fn path_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CurrentPath, &Position)>,
) {
    for (entity, mut path, position) in query.iter_mut() {
        // If the agent is at the current head of the path, pop it.
        // This handles both the starting node and arriving at an intermediate node.
        let mut arrived_at_node = false;
        if let Some(next_node) = path.nodes.front() {
            if next_node.0 == position.x && next_node.1 == position.y {
                arrived_at_node = true;
            }
        }
        if arrived_at_node {
            let popped_node = path.nodes.pop_front().unwrap(); // Safe to unwrap due to check above
            debug!("Entity {entity:?} arrived at path node {popped_node:?}");
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
