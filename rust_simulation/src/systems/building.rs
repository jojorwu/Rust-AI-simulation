use crate::components::{Chest, Inventory, Position, WantsToBuild};
use crate::ecs::World;
use crate::errors::SimulationError;
use crate::events::{Event, EventBus};
use crate::systems::{Resource, System, SystemResources};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct BuildingSystem;

impl System for BuildingSystem {
    fn name(&self) -> &'static str {
        "Building"
    }

    fn read_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::RecipeManager);
        resources
    }

    fn write_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::World);
        resources.insert(Resource::Map);
        resources.insert(Resource::EventBus);
        resources
    }

    fn run(&self, world: &mut World, resources: &mut SystemResources) -> Result<(), SimulationError> {
        let mut to_build = Vec::new();
        for entity in 0..world.entities.len() {
            if let Some(wants_to_build) = world.get_component::<WantsToBuild>(entity) {
                to_build.push((entity, wants_to_build.clone()));
            }
        }

        for (builder, wants_to_build) in &to_build {
            if let Some(builder_pos) = world.get_component::<Position>(*builder).copied() {
                let tile = &mut resources.map.grid[builder_pos.y as usize][builder_pos.x as usize];

                if tile.tile_type == '.' {
                    if let Some(inventory) = world.get_component_mut::<Inventory>(*builder) {
                        let required =
                            resources.recipe_manager.get_required_resources(&wants_to_build.structure_name, 1);
                        if inventory.has_resources(&required)
                            && inventory.remove_resources(&required) {
                                let built_structure = wants_to_build.structure_name.clone();

                                if built_structure == "chest" {
                                    let chest_entity = world.create_entity();
                                    world.add_component(chest_entity, builder_pos)?;
                                    world.add_component(
                                        chest_entity,
                                        Chest {
                                            inventory: Inventory::new(),
                                        },
                                    )?;
                                    tile.tile_type = 'C';
                                } else {
                                    tile.tile_type = match built_structure.as_str() {
                                        "foundation" => 'B',
                                        "wall" => '#',
                                        "doorway" => 'O',
                                        _ => 'X',
                                    };

                                    if built_structure == "foundation" {
                                        resources.event_bus
                                            .lock()
                                            .map_err(|e| {
                                                crate::errors::SimulationError::MutexLockError(
                                                    e.to_string(),
                                                )
                                            })?
                                            .publish(Event::FoundationBuilt {
                                                builder: *builder,
                                                position: builder_pos,
                                            });
                                    }
                                }
                            }
                    }
                }
            }
        }

        // Reset wants to build
        let builders: Vec<_> = to_build.iter().map(|(e, _)| *e).collect();
        for builder in builders {
            world.remove_component::<WantsToBuild>(builder);
        }
        Ok(())
    }
}
