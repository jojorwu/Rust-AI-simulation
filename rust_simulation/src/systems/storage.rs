use crate::components::{intents::WantsToStoreItem, Chest, Inventory};
use bevy_ecs::prelude::*;

pub fn storage_system(
    mut commands: Commands,
    mut storer_query: Query<(Entity, &mut Inventory, &WantsToStoreItem)>,
    mut chest_query: Query<&mut Chest>,
) {
    for (storer_entity, mut storer_inventory, wants_to_store) in storer_query.iter_mut() {
        // First, check if the target chest exists and get mutable access to it.
        if let Ok(mut chest) = chest_query.get_mut(wants_to_store.target_chest) {
            // If the chest exists, *then* try to remove the item from the storer.
            if storer_inventory.remove_item(&wants_to_store.item_name, wants_to_store.quantity) {
                // If removal was successful, add the item to the chest.
                chest
                    .inventory
                    .add_item(&wants_to_store.item_name, wants_to_store.quantity);
                // And then remove the intent.
                commands.entity(storer_entity).remove::<WantsToStoreItem>();
            }
        }
        // If the chest doesn't exist, we don't remove the intent. The agent will try again later.
    }
}
