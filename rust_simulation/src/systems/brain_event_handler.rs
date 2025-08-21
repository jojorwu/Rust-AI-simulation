use crate::components::BrainComponent;
use crate::ecs::World;
use crate::errors::SimulationError;
use crate::events::{Event, EventBus};
use crate::systems::{Resource, System, SystemResources};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct BrainEventHandlerSystem;

impl System for BrainEventHandlerSystem {
    fn name(&self) -> &'static str {
        "BrainEventHandler"
    }

    fn read_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::EventBus);
        resources
    }

    fn write_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::World);
        resources
    }

    fn run(&self, world: &mut World, resources: &mut SystemResources) -> Result<(), SimulationError> {
        let events = resources.event_bus
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?
            .take_events();
        for event in events {
            if let Event::FoundationBuilt { builder, position } = event {
                if let Some(brain_component) = world.get_component_mut::<BrainComponent>(builder) {
                    if brain_component.home_base.is_none() {
                        brain_component.home_base = Some(position);
                    }
                }
            }
        }
        Ok(())
    }
}
