use crate::components::{Position, Velocity};
use crate::map::Map;
use bevy_ecs::prelude::*;

use log::debug;

pub fn movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Position, &Velocity)>,
    map: Res<Map>,
) {
    for (entity, mut pos, vel) in query.iter_mut() {
        debug!("Movement system running for entity {:?} with velocity {:?}", entity, vel);
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
            map.remove_entity_from_spatial_map(entity, old_pos.x, old_pos.y);
            map.add_entity_to_spatial_map(entity, pos.x, pos.y);
        }

        // Remove the Velocity component, as it represents a one-time movement intent.
        commands.entity(entity).remove::<Velocity>();
    }
}
