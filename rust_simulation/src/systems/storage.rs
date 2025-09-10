use crate::components::{Chest, Inventory, WantsToStoreItem};
use bevy_ecs::prelude::*;

pub fn storage_system(
    mut commands: Commands,
    mut storer_query: Query<(Entity, &mut Inventory, &WantsToStoreItem)>,
    mut chest_query: Query<&mut Chest>,
) {
    for (storer_entity, mut storer_inventory, wants_to_store) in storer_query.iter_mut() {
        if let Ok(mut chest) = chest_query.get_mut(wants_to_store.target_chest) {
            let item_name = &wants_to_store.item_name;
            let quantity = wants_to_store.quantity;

            // Check if the chest has capacity for a new item stack.
            // If the item is already in the chest, we can add to the stack regardless of capacity.
            let has_capacity = chest.inventory.items.contains_key(item_name)
                || (chest.inventory.items.len() as u32) < chest.capacity;

            if has_capacity {
                if storer_inventory.remove_item(item_name, quantity) {
                    chest.inventory.add_item(item_name, quantity);
                }
            }
        }
        // Always remove the intent, whether it succeeded or failed, to prevent getting stuck.
        commands.entity(storer_entity).remove::<WantsToStoreItem>();
    }
}
