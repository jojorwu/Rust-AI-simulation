use crate::components::{Chest, Inventory, WantsToStoreItem};
use bevy_ecs::prelude::*;

pub fn storage_system(
    mut commands: Commands,
    mut storer_query: Query<(Entity, &mut Inventory, &WantsToStoreItem)>,
    mut chest_query: Query<&mut Chest>,
) {
    for (storer_entity, mut storer_inventory, wants_to_store) in storer_query.iter_mut() {
        if storer_inventory.remove_item(&wants_to_store.item_name, wants_to_store.quantity) {
            if let Ok(mut chest) = chest_query.get_mut(wants_to_store.target_chest) {
                chest.inventory.add_item(&wants_to_store.item_name, wants_to_store.quantity);
            }
        }
        commands.entity(storer_entity).remove::<WantsToStoreItem>();
    }
}
