use crate::components::{Inventory, Position, Resource as ResourceComponent, WantsToGather};
use crate::ecs::World;
use crate::errors::SimulationError;
use crate::systems::{Resource, System, SystemResources};
use std::collections::HashSet;

pub struct GatheringSystem;

impl System for GatheringSystem {
    fn name(&self) -> &'static str {
        "Gathering"
    }

    fn read_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::ItemRegistry);
        resources
    }

    fn write_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::World);
        resources
    }

    fn run(&self, world: &mut World, _resources: &mut SystemResources) -> Result<(), SimulationError> {
        let mut to_gather = Vec::new();
        for entity in 0..world.entities.len() {
            if let Some(wants_to_gather) = world.get_component::<WantsToGather>(entity) {
                to_gather.push((entity, wants_to_gather.target));
            }
        }

        for (gatherer, target) in to_gather {
            if let (Some(gatherer_pos), Some(target_pos)) = (
                world.get_component::<Position>(gatherer).copied(),
                world.get_component::<Position>(target).copied(),
            ) {
                let dx = (gatherer_pos.x as i32 - target_pos.x as i32).abs();
                let dy = (gatherer_pos.y as i32 - target_pos.y as i32).abs();

                if dx <= 1 && dy <= 1 {
                    let resource_name =
                        if let Some(resource) = world.get_component_mut::<ResourceComponent>(target) {
                            if resource.quantity > 0 {
                                resource.quantity -= 1;
                                Some(resource.name.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                    if let Some(name) = resource_name {
                        if let Some(inventory) = world.get_component_mut::<Inventory>(gatherer) {
                            inventory.add_item(&name, 1);
                        }
                    }
                }
            }
        }

        // Reset wants to gather
        for entity in 0..world.entities.len() {
            world.remove_component::<WantsToGather>(entity);
        }

        Ok(())
    }
}
