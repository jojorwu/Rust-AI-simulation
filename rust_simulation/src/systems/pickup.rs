use crate::components::{
    DroppedItem, Inventory, IsPickingUp, PickupClaimed, Position, WantsToPickup,
};
use crate::map::Map;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

pub fn claim_item_system(
    mut commands: Commands,
    map: Res<Map>,
    // A query for all entities that want to pick something up.
    picker_query: Query<(Entity, &Position), With<WantsToPickup>>,
    // A query to identify items that have not yet been claimed.
    item_query: Query<Entity, (With<DroppedItem>, Without<PickupClaimed>)>,
) {
    let mut claimed_items_this_tick = HashSet::new();

    for (picker_entity, pos) in picker_query.iter() {
        let mut successfully_claimed = false;
        if let Some(entities_on_tile) = map.get_entities_at(pos.x, pos.y) {
            for &item_entity in &entities_on_tile {
                // Check if the entity on the tile is an available item and hasn't been claimed this tick.
                if item_query.get(item_entity).is_ok()
                    && !claimed_items_this_tick.contains(&item_entity)
                {
                    // Claim the item for this agent.
                    commands.entity(item_entity).insert(PickupClaimed);
                    commands
                        .entity(picker_entity)
                        .insert(IsPickingUp { item: item_entity });

                    // Mark as claimed for this tick to prevent other agents in this same system run.
                    claimed_items_this_tick.insert(item_entity);

                    successfully_claimed = true;
                    // Agent has claimed an item, so we can stop looking for this agent.
                    break;
                }
            }
        }
        // If the agent successfully claimed an item, consume the generic "WantsToPickup" intent.
        // If not, they will try again next tick.
        if successfully_claimed {
            commands.entity(picker_entity).remove::<WantsToPickup>();
        }
    }
}

pub fn pickup_system(
    mut commands: Commands,
    map: Res<Map>,
    // A query for agents that are in the process of picking up a claimed item.
    mut picker_query: Query<(Entity, &mut Inventory, &IsPickingUp, &Position)>,
    // A query to get the details of the dropped items.
    item_query: Query<(&DroppedItem, &Position)>,
) {
    for (picker_entity, mut inventory, is_picking_up, picker_pos) in picker_query.iter_mut() {
        let item_entity = is_picking_up.item;

        // Get the item's details, if it still exists.
        if let Ok((item, item_pos)) = item_query.get(item_entity) {
            // Check if the picker is on the same tile as the item.
            if picker_pos == item_pos {
                // Add the item to the picker's inventory.
                inventory.add_item(&item.item_name, item.quantity);

                // The item has been picked up, so it no longer exists in the world.
                commands.entity(item_entity).despawn();
                map.remove_entity_from_spatial_map(item_entity, item_pos.x, item_pos.y);
            }
        }

        // Whether the pickup succeeded or the item disappeared, the intent is consumed.
        commands.entity(picker_entity).remove::<IsPickingUp>();
    }
}
