use crate::components::{Chest, Inventory};
use crate::events::Event;
use crate::map::{CHUNK_SIZE, Map};
use bevy_ecs::prelude::*;
use log::error;

pub fn map_modification_system(
    mut commands: Commands,
    mut event_reader: EventReader<Event>,
    map: Res<Map>,
) {
    for event in event_reader.read() {
        if let Event::BuildRequest {
            builder: _,
            structure,
            position,
        } = event
        {
            if let Some((chunk_x, chunk_y)) = map.get_chunk_index(position.x, position.y) {

                let chunk_lock = map.chunks[chunk_y][chunk_x].lock();
                let mut chunk = match chunk_lock {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        error!("Mutex was poisoned. Recovering. Error: {poisoned:?}");
                        poisoned.into_inner()
                    }
                };

                let local_x = (position.x % CHUNK_SIZE) as usize;
                let local_y = (position.y % CHUNK_SIZE) as usize;

                // Re-check the tile to prevent race conditions
                if chunk.tiles[local_y][local_x].tile_type != '.' {
                    // Tile is already occupied, so we skip this build request.
                    continue;
                }

                match structure.as_str() {
                    "chest" => {
                        // Spawn a chest entity
                        let chest_entity = commands.spawn((
                            *position,
                            Chest {
                                inventory: Inventory::new(),
                            },
                        )).id();

                        // Mutably borrow chunk to update the spatial map
                        chunk
                            .spatial_map
                            .entry((local_x as u32, local_y as u32))
                            .or_default()
                            .push(chest_entity);

                        // Now that the first mutable borrow is done, get the tile and change it.
                        let tile = &mut chunk.tiles[local_y][local_x];
                        tile.tile_type = 'C';
                    }
                    "foundation" => {
                        let tile = &mut chunk.tiles[local_y][local_x];
                        tile.tile_type = 'B';
                    }
                    "wall" => {
                        let tile = &mut chunk.tiles[local_y][local_x];
                        tile.tile_type = '#';
                    }
                    "doorway" => {
                        let tile = &mut chunk.tiles[local_y][local_x];
                        tile.tile_type = 'O';
                    }
                    _ => {
                        let tile = &mut chunk.tiles[local_y][local_x];
                        tile.tile_type = 'X'; // Default for unknown structures
                    }
                }
            }
        }
    }
}
