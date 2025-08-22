use crate::components::{intents::IntendsToBuild, Chest, Inventory, Position};
use crate::events::Event;
use crate::map::{Map, CHUNK_SIZE};
use crate::RecipeManagerResource;
use bevy_ecs::prelude::*;

pub fn building_system(
    mut commands: Commands,
    mut builder_query: Query<(Entity, &Position, &mut Inventory, &IntendsToBuild)>,
    mut event_writer: EventWriter<Event>,
    map: Res<Map>,
    recipe_manager: Res<RecipeManagerResource>,
) {
    let recipe_manager = &recipe_manager.0;

    for (builder_entity, pos, mut inventory, intends_to_build) in builder_query.iter_mut() {
        let required =
            recipe_manager.get_required_resources(&intends_to_build.0, 1);

        // Check if the builder has the required resources
        if !inventory.has_resources(&required) {
            commands.entity(builder_entity).remove::<IntendsToBuild>();
            continue;
        }

        if let Some((chunk_x, chunk_y)) = map.get_chunk_index(pos.x, pos.y) {
            let mut chunk = map.chunks[chunk_y][chunk_x].lock().unwrap();
            let local_x = (pos.x % CHUNK_SIZE) as usize;
            let local_y = (pos.y % CHUNK_SIZE) as usize;
            let tile = &mut chunk.tiles[local_y][local_x];

            // Check if the tile is suitable for building (e.g., it's empty ground)
            // For now, we assume the agent builds on the tile it is standing on.
            // A more advanced system would allow targeting adjacent tiles.
            if tile.tile_type == '.' {
                // Consume resources
                if inventory.remove_resources(&required) {
                    let built_structure = intends_to_build.0.clone();

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
        // In a more complex system, this might remain if, for example, the agent
        // needed to move to a valid building location first.
        commands.entity(builder_entity).remove::<IntendsToBuild>();
    }
}
