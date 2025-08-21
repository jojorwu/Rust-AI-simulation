use crate::components::{Position, Velocity};
use crate::ecs::World;
use crate::errors::SimulationError;
use crate::systems::{Resource, System, SystemResources};
use std::collections::HashSet;

pub struct MovementSystem;

impl System for MovementSystem {
    fn name(&self) -> &'static str {
        "Movement"
    }

    fn write_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::World);
        resources.insert(Resource::Map);
        resources
    }

    fn run(&self, world: &mut World, resources: &mut SystemResources) -> Result<(), SimulationError> {
        let entities_with_velocity: Vec<_> = world
            .entities
            .iter()
            .filter_map(|&entity| {
                world
                    .get_component::<Velocity>(entity)
                    .map(|vel| (entity, *vel))
            })
            .collect();

        for (entity, vel) in entities_with_velocity {
            if let Some(pos) = world.get_component_mut::<Position>(entity) {
                // Remove from old position in spatial map
                resources.map
                    .spatial_map
                    .entry((pos.x, pos.y))
                    .and_modify(|v| v.retain(|&e| e != entity));

                pos.x = (pos.x as i32 + vel.dx) as u32;
                pos.y = (pos.y as i32 + vel.dy) as u32;

                // Add to new position in spatial map
                resources.map
                    .spatial_map
                    .entry((pos.x, pos.y))
                    .or_default()
                    .push(entity);
            }
        }

        // Reset velocities
        let entities_with_velocity: Vec<_> = world.entities.iter().copied().collect();
        for entity in entities_with_velocity {
            world.remove_component::<Velocity>(entity);
        }

        Ok(())
    }
}
