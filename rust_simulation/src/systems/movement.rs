use crate::components::{Position, Velocity};
use crate::map::Map;
use bevy::ecs::system::ParallelCommands;
use bevy_ecs::prelude::*;
use rayon::prelude::*;

use log::debug;

pub fn movement_system(
    commands: ParallelCommands,
    mut query: Query<(Entity, &mut Position, &Velocity)>,
    map: Res<Map>,
) {
    query.par_iter_mut().for_each(|(entity, mut pos, vel)| {
        debug!(
            "Movement system running for entity {entity:?} with velocity {vel:?}"
        );
        // Store the old position before updating.
        let old_pos = *pos;

        // Calculate the new position.
        // This assumes the velocity is valid and doesn't lead to out-of-bounds.
        // A more robust system would check boundaries and collisions.
        let new_x = (pos.x as i32 + vel.dx) as u32;
        let new_y = (pos.y as i32 + vel.dy) as u32;

        // Basic boundary check
        if new_x < map.width && new_y < map.height {
            // Update the entity's position component.
            pos.x = new_x;
            pos.y = new_y;

            // Update the spatial map in the corresponding map chunks.
            // This is safe to do in parallel because the Map uses Mutexes on its chunks.
            map.remove_entity_from_spatial_map(entity, old_pos.x, old_pos.y);
            map.add_entity_to_spatial_map(entity, pos.x, pos.y);
        }

        // Use command_scope for safe parallel command buffering.
        commands.command_scope(|mut c| {
            // Remove the Velocity component, as it represents a one-time movement intent.
            c.entity(entity).remove::<Velocity>();
        });
    });
}
