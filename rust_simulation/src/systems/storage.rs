use crate::components::{Chest, Inventory, WantsToStoreItem};
use crate::ecs::World;
use crate::errors::SimulationError;
use crate::systems::{Resource, System, SystemResources};
use std::collections::HashSet;

pub struct StorageSystem;

impl System for StorageSystem {
    fn name(&self) -> &'static str {
        "Storage"
    }

    fn write_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::World);
        resources
    }

    fn run(&self, world: &mut World, _resources: &mut SystemResources) -> Result<(), SimulationError> {
        let mut to_store = Vec::new();
        for entity in 0..world.entities.len() {
            if let Some(wants_to_store) = world.get_component::<WantsToStoreItem>(entity) {
                to_store.push((entity, wants_to_store.clone()));
            }
        }

        let mut successful_transfers = Vec::new();

        // Step 1: Check for validity and remove from storer
        for (storer, wants_to_store) in &to_store {
            if let Some(storer_inventory) = world.get_component_mut::<Inventory>(*storer) {
                if storer_inventory.remove_item(&wants_to_store.item_name, wants_to_store.quantity) {
                    // If removal was successful, queue the item for addition to the chest
                    successful_transfers.push(wants_to_store.clone());
                }
            }
        }

        // Step 2: Add to chest
        for transfer in successful_transfers {
            if let Some(chest_component) = world.get_component_mut::<Chest>(transfer.target_chest) {
                chest_component
                    .inventory
                    .add_item(&transfer.item_name, transfer.quantity);
            }
        }

        // Reset wants to store
        for (storer, _) in to_store {
            world.remove_component::<WantsToStoreItem>(storer);
        }

        Ok(())
    }
}
