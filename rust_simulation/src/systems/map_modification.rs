use crate::components::{Chest, Inventory};
use crate::events::Event;
use crate::map::{CHUNK_SIZE, Map};
use bevy_ecs::prelude::*;

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
                let mut chunk = map.chunks[chunk_y][chunk_x].lock().unwrap();
                let local_x = (position.x % CHUNK_SIZE) as usize;
                let local_y = (position.y % CHUNK_SIZE) as usize;
                let tile = &mut chunk.tiles[local_y][local_x];

                match structure.as_str() {
                    "chest" => {
                        // Spawn a chest entity and change the tile type
                        commands.spawn((
                            *position,
                            Chest {
                                inventory: Inventory::new(),
                            },
                        ));
                        tile.tile_type = 'C';
                    }
                    "foundation" => {
                        tile.tile_type = 'B';
                    }
                    "wall" => tile.tile_type = '#',
                    "doorway" => tile.tile_type = 'O',
                    _ => tile.tile_type = 'X', // Default for unknown structures
                }
            }
        }
    }
}
