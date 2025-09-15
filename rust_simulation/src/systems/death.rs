use crate::animals::pig::Pig;
use crate::components::{DroppedItem, Inventory, Position};
use crate::events::Event;
use crate::map::Map;
use bevy_ecs::prelude::*;

// This system is now responsible only for the final cleanup of a dead entity.
pub fn death_cleanup_system(
    mut commands: Commands,
    mut event_reader: EventReader<Event>,
    position_query: Query<&Position>,
    map: Res<Map>,
) {
    for event in event_reader.read() {
        if let Event::EntityDied(entity) = event {
            // Remove entity from spatial map if it has a position
            if let Ok(pos) = position_query.get(*entity) {
                map.remove_entity_from_spatial_map(*entity, pos.x, pos.y);
            }

            // Despawn the dead entity from the world
            if let Some(mut entity_commands) = commands.get_entity(*entity) {
                entity_commands.despawn();
            }
        }
    }
}

// New system for handling pig-specific death logic
pub fn pig_death_handler(
    mut commands: Commands,
    mut event_reader: EventReader<Event>,
    pig_query: Query<(Entity, &Position), With<Pig>>,
    map: Res<Map>,
) {
    for event in event_reader.read() {
        if let Event::EntityDied(entity) = event {
            if let Ok((_, pos)) = pig_query.get(*entity) {
                // This is a pig, so drop meat
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
}

// New system for handling generic inventory drops on death
pub fn inventory_drop_on_death_system(
    mut commands: Commands,
    mut event_reader: EventReader<Event>,
    query: Query<(Entity, &Position, &Inventory)>,
    map: Res<Map>,
) {
    for event in event_reader.read() {
        if let Event::EntityDied(entity) = event {
            if let Ok((_, pos, inventory)) = query.get(*entity) {
                // If the entity had an inventory, drop its items
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
    }
}
