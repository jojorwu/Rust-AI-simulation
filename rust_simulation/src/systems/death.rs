use crate::animals::pig::Pig;
use crate::components::{DroppedItem, Inventory, Position};
use crate::events::Event;
use crate::map::Map;
use bevy_ecs::prelude::*;

pub fn death_system(
    mut commands: Commands,
    mut event_reader: EventReader<Event>,
    position_query: Query<&Position>,
    pig_query: Query<&Position, With<Pig>>,
    inventory_query: Query<&Inventory>, // Add the new query here
    map: Res<Map>,
) {
    for event in event_reader.read() {
        if let Event::EntityDied(entity) = event {
            if let Ok(pos) = position_query.get(*entity) {
                map.remove_entity_from_spatial_map(*entity, pos.x, pos.y);

                // If the entity was a pig, drop meat
                if pig_query.get(*entity).is_ok() {
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

                // If the entity had an inventory, drop its items
                if let Ok(inventory) = inventory_query.get(*entity) {
                    for (item_name, &quantity) in &inventory.items {
                        if quantity > 0 {
                            let dropped_item_entity = commands
                                .spawn((
                                    DroppedItem {
                                        item_name: item_name.clone(),
                                        quantity,
                                    },
                                    *pos,
                                ))
                                .id();
                            map.add_entity_to_spatial_map(dropped_item_entity, pos.x, pos.y);
                        }
                    }
                }
            }

            // Despawn the dead entity from the world
            commands.entity(*entity).despawn();
        }
    }
}
