use crate::components::{DroppedItem, Inventory, Position};
use crate::events::Event;
use crate::map::Map;
use bevy_ecs::prelude::*;

pub fn death_system(
    mut commands: Commands,
    mut event_reader: EventReader<Event>,
    query: Query<(Option<&Position>, Option<&Inventory>)>,
    map: Res<Map>,
) {
    for event in event_reader.read() {
        if let Event::EntityDied(entity) = event {
            if let Ok((pos_opt, inv_opt)) = query.get(*entity) {
                if let Some(pos) = pos_opt {
                    // Drop items from inventory
                    if let Some(inventory) = inv_opt {
                        for (item_name, quantity) in &inventory.items {
                            if *quantity > 0 {
                                let dropped_item_entity = commands
                                    .spawn((
                                        DroppedItem {
                                            item_name: item_name.clone(),
                                            quantity: *quantity,
                                        },
                                        *pos,
                                    ))
                                    .id();
                                map.add_entity_to_spatial_map(
                                    dropped_item_entity,
                                    pos.x,
                                    pos.y,
                                );
                            }
                        }
                    }
                    // Remove entity from spatial map
                    map.remove_entity_from_spatial_map(*entity, pos.x, pos.y);
                }
            }

            // Despawn the dead entity from the world
            commands.entity(*entity).despawn();
        }
    }
}
