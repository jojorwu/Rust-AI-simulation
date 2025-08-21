use crate::components::{DroppedItem, Position};
use crate::ecs::World;
use crate::errors::SimulationError;
use crate::events::{Event, EventBus};
use crate::systems::{Resource, System, SystemResources};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct DeathSystem;

impl System for DeathSystem {
    fn name(&self) -> &'static str {
        "Death"
    }

    fn read_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::EventBus);
        resources
    }

    fn write_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::World);
        resources.insert(Resource::Map);
        resources
    }

    fn run(&self, world: &mut World, resources: &mut SystemResources) -> Result<(), SimulationError> {
        let events = resources.event_bus
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?
            .take_events();
        for event in events {
            if let Event::EntityDied(entity) = event {
                if let Some(pos) = world.get_component::<Position>(entity).copied() {
                    // Remove the dead entity from the spatial map
                    resources.map
                        .spatial_map
                        .entry((pos.x, pos.y))
                        .and_modify(|v| v.retain(|&e| e != entity));

                    // Create a new entity for the dropped item
                    let dropped_item_entity = world.create_entity();
                    world.add_component(
                        dropped_item_entity,
                        DroppedItem {
                            item_name: "meat".to_string(),
                            quantity: 1,
                        },
                    )?;
                    world.add_component(dropped_item_entity, pos)?;

                    // Add the new dropped item to the spatial map
                    resources.map
                        .spatial_map
                        .entry((pos.x, pos.y))
                        .or_default()
                        .push(dropped_item_entity);
                }
                // Remove the dead entity from the world
                world.remove_entity(entity);
            }
        }
        Ok(())
    }
}
