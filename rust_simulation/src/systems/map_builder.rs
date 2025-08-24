use crate::events::Event;
use crate::map::Map;
use bevy_ecs::prelude::*;

pub fn map_builder_system(
    mut event_reader: EventReader<Event>,
    map: ResMut<Map>,
) {
    for event in event_reader.read() {
        if let Event::ChunkGenerated { position, tiles } = event {
            let (chunk_x, chunk_y) = *position;
            if let Some(chunk_mutex) = map.chunks.get(chunk_y as usize).and_then(|row| row.get(chunk_x as usize)) {
                let mut chunk = chunk_mutex.lock().unwrap();
                chunk.tiles = tiles.clone();
            }
        }
    }
}
