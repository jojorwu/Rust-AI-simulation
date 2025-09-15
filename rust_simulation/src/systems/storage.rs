use crate::components::{Chest, Inventory, Position, WantsToStoreItem};
use bevy_ecs::prelude::*;

pub fn storage_system(
    mut commands: Commands,
    mut storer_query: Query<(Entity, &mut Inventory, &WantsToStoreItem, &Position)>,
    mut chest_query: Query<(&mut Chest, &Position)>,
) {
    for (storer_entity, mut storer_inventory, wants_to_store, storer_pos) in
        storer_query.iter_mut()
    {
        // First, check if the target chest exists and get mutable access to it.
        if let Ok((mut chest, chest_pos)) = chest_query.get_mut(wants_to_store.target_chest) {
            // Check if the storer is adjacent to the chest.
            if storer_pos.distance(chest_pos) < 1.5 {
                // If the chest exists and is close, *then* try to remove the item from the storer.
                if storer_inventory.remove_item(&wants_to_store.item_name, wants_to_store.quantity)
                {
                    // If removal was successful, add the item to the chest.
                    chest
                        .inventory
                        .add_item(&wants_to_store.item_name, wants_to_store.quantity);
                }
            }
        }
        // Always remove the intent, whether it succeeded or failed, to prevent getting stuck.
        commands.entity(storer_entity).remove::<WantsToStoreItem>();
    }
}
