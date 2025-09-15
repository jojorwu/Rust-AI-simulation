use crate::components::{DroppedItem, Inventory, Position, WantsToPickup};
use crate::map::Map;
use bevy_ecs::prelude::*;

pub fn pickup_system(
    mut commands: Commands,
    map: Res<Map>,
    // A query for all entities that want to pick something up.
    mut picker_query: Query<(Entity, &Position, &mut Inventory), With<WantsToPickup>>,
    // A query to identify entities that are, in fact, dropped items.
    item_query: Query<&DroppedItem>,
) {
    for (picker_entity, pos, mut inventory) in picker_query.iter_mut() {
        if let Some(entities_on_tile) = map.get_entities_at(pos.x, pos.y) {
            for &item_entity in &entities_on_tile {
                // An entity cannot pick itself up.
                if item_entity == picker_entity {
                    continue;
                }

                // Check if the entity on the tile is a dropped item.
                if let Ok(item) = item_query.get(item_entity) {
                    // Add the item to the picker's inventory.
                    inventory.add_item(&item.item_name, item.quantity);

                    // The item has been picked up, so it no longer exists in the world.
                    // We must despawn it.
                    commands.entity(item_entity).despawn();

                    // We also need to remove it from the spatial map to keep it consistent.
                    // Although despawning will make it disappear from queries next frame,
                    // not removing it from the map could cause other systems in the same frame
                    // to see a "ghost" entity.
                    map.remove_entity_from_spatial_map(item_entity, pos.x, pos.y);

                    // For simplicity, we assume an entity can only pick up one item stack per tick.
                    // We can break here if that's the desired behavior.
                    break;
                }
            }
        }

        // The entity has attempted to pick up, so we remove the component.
        commands.entity(picker_entity).remove::<WantsToPickup>();
    }
}
