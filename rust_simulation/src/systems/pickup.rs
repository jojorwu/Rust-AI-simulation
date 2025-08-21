use crate::components::{DroppedItem, Inventory, Position, WantsToPickup};
use crate::ecs::World;
use crate::errors::SimulationError;
use crate::systems::{Resource, System, SystemResources};
use std::collections::HashSet;

pub struct PickupSystem;

impl System for PickupSystem {
    fn name(&self) -> &'static str {
        "Pickup"
    }

    fn read_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::ItemRegistry);
        resources
    }

    fn write_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::World);
        resources.insert(Resource::Map);
        resources
    }

    fn run(&self, world: &mut World, resources: &mut SystemResources) -> Result<(), SimulationError> {
        let to_pickup: Vec<_> = world
            .entities
            .iter()
            .copied()
            .filter(|&entity| world.get_component::<WantsToPickup>(entity).is_some())
            .collect();

        for picker_upper in to_pickup {
            if let Some(picker_upper_pos) = world.get_component::<Position>(picker_upper).copied() {
                let mut items_to_remove = Vec::new();
                let mut items_to_add = Vec::new();

                if let Some(entities_on_tile) = resources.map
                    .spatial_map
                    .get(&(picker_upper_pos.x, picker_upper_pos.y))
                {
                    for &entity in entities_on_tile {
                        if let Some(item) = world.get_component::<DroppedItem>(entity) {
                            items_to_add.push((picker_upper, item.clone()));
                            items_to_remove.push(entity);
                        }
                    }
                }

                for (picker_upper, item) in items_to_add {
                    if let Some(inventory) = world.get_component_mut::<Inventory>(picker_upper) {
                        inventory.add_item(&item.item_name, item.quantity);
                    }
                }

                for entity in items_to_remove.iter() {
                    if let Some(pos) = world.get_component::<Position>(*entity) {
                        resources.map
                            .spatial_map
                            .entry((pos.x, pos.y))
                            .and_modify(|v| v.retain(|&e| e != *entity));
                    }
                    world.remove_entity(*entity);
                }
            }
        }

        // Reset wants to pickup
        for entity in world.entities.clone() {
            world.remove_component::<WantsToPickup>(entity);
        }

        Ok(())
    }
}
