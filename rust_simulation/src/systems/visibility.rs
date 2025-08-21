use crate::components::{Player, Position};
use crate::ecs::World;
use crate::errors::SimulationError;
use crate::fov;
use crate::map::TileState;
use crate::systems::{Resource, System, SystemResources};
use std::collections::HashSet;

pub struct VisibilitySystem;

impl System for VisibilitySystem {
    fn name(&self) -> &'static str {
        "Visibility"
    }

    fn read_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::Map);
        resources
    }

    fn write_resources(&self) -> HashSet<Resource> {
        let mut resources = HashSet::new();
        resources.insert(Resource::World);
        resources
    }

    fn run(&self, world: &mut World, resources: &mut SystemResources) -> Result<(), SimulationError> {
        for entity in 0..world.entities.len() {
            let player_pos = match world.get_component::<Position>(entity) {
                Some(pos) => *pos,
                None => continue,
            };

            if let Some(player) = world.get_component_mut::<Player>(entity) {
                // Step 1: Set all currently visible tiles to explored.
                for y in 0..player.mental_map.height {
                    for x in 0..player.mental_map.width {
                        if player.mental_map.grid[y as usize][x as usize] == TileState::Visible {
                            player.mental_map.grid[y as usize][x as usize] = TileState::Explored;
                        }
                    }
                }

                // Step 2: Calculate the new field of view.
                let fov_radius = if resources.is_day { 8 } else { 4 };
                let visible_tiles = fov::field_of_view(&player_pos, fov_radius, resources.map);

                // Step 3: Mark all tiles in the FOV as visible.
                for pos in visible_tiles.iter() {
                    player.mental_map.grid[pos.y as usize][pos.x as usize] = TileState::Visible;
                }
            }
        }
        Ok(())
    }
}
