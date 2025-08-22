use crate::components::{DroppedItem, Position};
use crate::events::Event;
use crate::map::Map;
use bevy_ecs::prelude::*;

pub fn death_system(
    mut commands: Commands,
    mut event_reader: EventReader<Event>,
    query: Query<&Position>,
    map: Res<Map>,
) {
    for event in event_reader.read() {
        if let Event::EntityDied(entity) = event {
            if let Ok(pos) = query.get(*entity) {
                // When an entity dies, it's removed from the world. We also need to update
                // the spatial map in the corresponding map chunk.
                map.remove_entity_from_spatial_map(*entity, pos.x, pos.y);

                // Create a new entity for the dropped item (e.g., meat)
                let dropped_item_entity = commands
                    .spawn((
                        DroppedItem {
                            item_name: "meat".to_string(),
                            quantity: 1,
                        },
                        *pos,
                    ))
                    .id();

                // Add the new dropped item to the spatial map
                map.add_entity_to_spatial_map(dropped_item_entity, pos.x, pos.y);
            }

            // Despawn the dead entity from the world
            commands.entity(*entity).despawn();
        }
    }
}
