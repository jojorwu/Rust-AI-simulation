use crate::components::{Chest, Inventory, Position, WantsToBuild};
use crate::events::Event;
use crate::lib::RecipeManagerResource;
use crate::map::{Map, CHUNK_SIZE};
use bevy_ecs::prelude::*;

pub fn building_system(
    mut commands: Commands,
    mut builder_query: Query<(Entity, &Position, &mut Inventory, &WantsToBuild)>,
    mut event_writer: EventWriter<Event>,
    map: Res<Map>,
    recipe_manager: Res<RecipeManagerResource>,
) {
    let recipe_manager = &recipe_manager.0;

    for (builder_entity, pos, mut inventory, wants_to_build) in builder_query.iter_mut() {
        let required = recipe_manager.get_required_resources(&wants_to_build.structure_name, 1);

        // Check if the builder has the required resources
        if !inventory.has_resources(&required) {
            commands.entity(builder_entity).remove::<WantsToBuild>();
            continue;
        }

        if let Some((chunk_x, chunk_y)) = map.get_chunk_index(pos.x, pos.y) {
            let mut chunk = map.chunks[chunk_y][chunk_x].lock().unwrap();
            let local_x = (pos.x % CHUNK_SIZE) as usize;
            let local_y = (pos.y % CHUNK_SIZE) as usize;
            let tile = &mut chunk.tiles[local_y][local_x];

            // Check if the tile is suitable for building (e.g., it's empty ground)
            if tile.tile_type == '.' {
                // Consume resources
                if inventory.remove_resources(&required) {
                    let built_structure = wants_to_build.structure_name.clone();

                    match built_structure.as_str() {
                        "chest" => {
                            // Spawn a chest entity and change the tile type
                            commands.spawn((
                                *pos,
                                Chest {
                                    inventory: Inventory::new(),
                                },
                            ));
                            tile.tile_type = 'C';
                        }
                        "foundation" => {
                            tile.tile_type = 'B';
                            event_writer.send(Event::FoundationBuilt {
                                builder: builder_entity,
                                position: *pos,
                            });
                        }
                        "wall" => tile.tile_type = '#',
                        "doorway" => tile.tile_type = 'O',
                        _ => tile.tile_type = 'X', // Default for unknown structures
                    }
                }
            }
        }

        // Remove the intent to build, whether successful or not.
        commands.entity(builder_entity).remove::<WantsToBuild>();
    }
}
