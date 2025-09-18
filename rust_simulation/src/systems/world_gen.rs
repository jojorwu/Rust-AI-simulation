use crate::events::Event;
use crate::map::{Map, Tile, CHUNK_SIZE};
use bevy_ecs::prelude::*;
use std::sync::Arc;

pub fn world_gen_system(mut event_writer: EventWriter<Event>, map: Res<Map>) {
    for y in 0..map.height_in_chunks() {
        for x in 0..map.width_in_chunks() {
            let mut tiles = vec![vec![Tile::new('.', "grassland".to_string()); CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
            event_writer.send(Event::ChunkGenerated {
                position: (x, y),
                tiles: Arc::new(tiles),
            });
        }
    }
}
