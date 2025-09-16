use crate::animals::pig::Pig;
use crate::components::{DroppedItem, Position};
use crate::events::Event;
use crate::map::Map;
use bevy_ecs::prelude::*;

pub fn death_system(
    mut commands: Commands,
    mut event_reader: EventReader<Event>,
    pig_query: Query<&Pig>, // We only need to check for the component, not the position
    map: Res<Map>,
) {
    for event in event_reader.read() {
        if let Event::EntityDied { entity, position } = event {
            map.remove_entity_from_spatial_map(*entity, position.x, position.y);

            if pig_query.get(*entity).is_ok() {
                // This is a pig, so drop meat
                let dropped_item_entity = commands
                    .spawn((
                        DroppedItem {
                            item_name: "meat".to_string(),
                            quantity: 1,
                        },
                        *position,
                    ))
                    .id();
                map.add_entity_to_spatial_map(dropped_item_entity, position.x, position.y);
            }

            // Despawn the dead entity from the world
            commands.entity(*entity).despawn();
        }
    }
}
