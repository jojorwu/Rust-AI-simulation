use crate::components::{Health, WantsToAttack};
use crate::ecs::World;
use crate::errors::SimulationError;
use crate::events::{Event, EventBus};
use crate::systems::{Resource, System, SystemResources};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct CombatSystem;

impl System for CombatSystem {
    fn name(&self) -> &'static str {
        "Combat"
    }

    fn write_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::World);
        resources.insert(Resource::EventBus);
        resources
    }

    fn run(&self, world: &mut World, resources: &mut SystemResources) -> Result<(), SimulationError> {
        let mut to_attack = Vec::new();
        for entity in 0..world.entities.len() {
            if let Some(wants_to_attack) = world.get_component::<WantsToAttack>(entity) {
                to_attack.push((entity, wants_to_attack.target));
            }
        }

        for (_attacker, target) in to_attack {
            let damage = 10; // Placeholder
            let mut target_dead = false;
            if let Some(health) = world.get_component_mut::<Health>(target) {
                health.current -= damage;
                if health.current <= 0 {
                    target_dead = true;
                }
            }

            if target_dead {
                resources.event_bus
                    .lock()
                    .map_err(|e| SimulationError::MutexLockError(e.to_string()))?
                    .publish(Event::EntityDied(target));
            }
        }

        // Reset wants to attack
        for entity in 0..world.entities.len() {
            world.remove_component::<WantsToAttack>(entity);
        }
        Ok(())
    }
}
