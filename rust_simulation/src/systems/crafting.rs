use crate::components::{Inventory, WantsToCraft};
use crate::ecs::World;
use crate::errors::SimulationError;
use crate::systems::{Resource, System, SystemResources};
use std::collections::HashSet;

pub struct CraftingSystem;

impl System for CraftingSystem {
    fn name(&self) -> &'static str {
        "Crafting"
    }

    fn read_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::RecipeManager);
        resources.insert(Resource::ItemRegistry);
        resources
    }

    fn write_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::World);
        resources
    }

    fn run(&self, world: &mut World, resources: &mut SystemResources) -> Result<(), SimulationError> {
        let mut to_craft = Vec::new();
        for entity in 0..world.entities.len() {
            if let Some(wants_to_craft) = world.get_component::<WantsToCraft>(entity) {
                to_craft.push((entity, wants_to_craft.clone()));
            }
        }

        for (crafter, wants_to_craft) in to_craft {
            let required_resources =
                resources.recipe_manager.get_required_resources(&wants_to_craft.item_name, 1);
            if let Some(inventory) = world.get_component_mut::<Inventory>(crafter) {
                if inventory.has_resources(&required_resources)
                    && inventory.remove_resources(&required_resources) {
                        inventory.add_item(&wants_to_craft.item_name, 1);
                    }
            }
        }

        // Reset wants to craft
        for entity in 0..world.entities.len() {
            world.remove_component::<WantsToCraft>(entity);
        }

        Ok(())
    }
}
