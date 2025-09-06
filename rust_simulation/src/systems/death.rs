use crate::animals::pig::Pig;
use crate::components::{DroppedItem, Position};
use crate::events::Event;
use crate::map::Map;
use bevy_ecs::prelude::*;

pub fn death_system(
    mut commands: Commands,
    mut event_reader: EventReader<Event>,
    // Use a tuple to query for optional components
    query: Query<(Option<&Position>, Option<&Pig>)>,
    map: Res<Map>,
) {
    for event in event_reader.read() {
        if let Event::EntityDied(entity) = event {
            if let Ok((pos_option, pig_option)) = query.get(*entity) {
                // If the entity had a position, handle map cleanup and item drops
                if let Some(pos) = pos_option {
                    map.remove_entity_from_spatial_map(*entity, pos.x, pos.y);

                    // If the entity was a pig, drop meat
                    if pig_option.is_some() {
                        let dropped_item_entity = commands
                            .spawn((
                                DroppedItem {
                                    item_name: "meat".to_string(),
                                    quantity: 1,
                                },
                                *pos,
                            ))
                            .id();
                        map.add_entity_to_spatial_map(dropped_item_entity, pos.x, pos.y);
                    }
                }
            }

            // Always despawn the dead entity, regardless of its components
            commands.entity(*entity).despawn();
        }
    }
}
