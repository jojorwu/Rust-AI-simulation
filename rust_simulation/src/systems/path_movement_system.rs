use crate::components::{
    path::{CurrentPath, PathRequest},
    Position, Velocity,
};
use bevy_ecs::prelude::*;
use log::debug;

const STUCK_THRESHOLD: u32 = 5;

type PathMovementQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut CurrentPath,
        &'static Position,
        Option<&'static Velocity>,
    )
>;

pub fn path_movement_system(mut commands: Commands, mut query: PathMovementQuery) {
    for (entity, mut path, position, velocity) in query.iter_mut() {
        // The ultimate destination is the last node in the path.
        let ultimate_goal = path.nodes.back().cloned();

        // If the agent is at the current head of the path, pop it and reset the stuck timer.
        if let Some(&next_node) = path.nodes.front() {
            if next_node.0 == position.x && next_node.1 == position.y {
                path.nodes.pop_front();
                path.stuck_timer = 0;
                debug!("Entity {entity:?} arrived at path node {next_node:?}, resetting stuck timer.");
            }
        }

        // After potentially popping, check the next node.
        if let Some(&target_node) = path.nodes.front() {
            // If the agent is not at the next node but has a velocity, it means it tried to move
            // last frame but failed. Increment the stuck timer.
            if velocity.is_some() {
                path.stuck_timer += 1;
                debug!("Entity {entity:?} may be stuck. Timer: {}", path.stuck_timer);
            }

            // Check if the agent is officially stuck.
            if path.stuck_timer > STUCK_THRESHOLD {
                debug!("Entity {entity:?} is stuck! Clearing path and requesting a new one.");
                // Remove the path and any velocity to stop movement.
                commands.entity(entity).remove::<(CurrentPath, Velocity)>();

                // Request a new path to the original destination.
                if let Some(goal) = ultimate_goal {
                    commands.entity(entity).insert(PathRequest {
                        start: (position.x, position.y),
                        goal,
                    });
                }
                // Skip the rest of the logic for this entity.
                continue;
            }

            // If not stuck, set velocity towards the next node.
            let dx = target_node.0 as i32 - position.x as i32;
            let dy = target_node.1 as i32 - position.y as i32;
            commands.entity(entity).insert(Velocity { dx, dy });
        } else {
            // No nodes left, the path is complete.
            debug!("Entity {entity:?} finished its path.");
            commands.entity(entity).remove::<(CurrentPath, Velocity)>();
        }
    }
}
