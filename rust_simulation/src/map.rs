//! This module handles the representation of the game world, divided into chunks for concurrent access.

use bevy_ecs::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

use crate::errors::SimulationError;

pub const CHUNK_SIZE: u32 = 16;

/// Represents the visibility state of a tile from a player's perspective.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TileState {
    Unseen,
    Explored,
    Visible,
}

/// Represents a single tile on the game map.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tile {
    pub tile_type: char,
    pub biome: String,
    pub original_tile_type: char,
    pub health: Option<f64>,
}

impl Tile {
    pub fn new(tile_type: char, biome: String) -> Self {
        Tile {
            tile_type,
            biome,
            original_tile_type: tile_type,
            health: None,
        }
    }
}

/// A chunk of the map, containing its own tiles and a spatial map for entities.
#[derive(Debug, Clone)]
pub struct MapChunk {
    pub tiles: Vec<Vec<Tile>>,
    pub spatial_map: HashMap<(u32, u32), Vec<Entity>>,
}

impl Default for MapChunk {
    fn default() -> Self {
        Self::new()
    }
}

impl MapChunk {
    pub fn new() -> Self {
        MapChunk {
            tiles: vec![
                vec![Tile::new(' ', "none".to_string()); CHUNK_SIZE as usize];
                CHUNK_SIZE as usize
            ],
            spatial_map: HashMap::new(),
        }
    }
}

/// Represents a biome type with its associated properties.
#[derive(Debug, Deserialize, Clone)]
pub struct Biome {
    pub name: String,
    pub tile_type: char,
    pub height_range: [f64; 2],
}

/// Represents a resource that can be found in the world.
#[derive(Debug, Deserialize, Clone)]
pub struct ResourceDef {
    pub name: String,
    pub biomes: Vec<String>,
    pub density: f64,
    #[serde(default)]
    pub huntable: bool,
}

/// The main map resource, containing a grid of map chunks.
#[derive(Resource, Clone)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub chunks: Vec<Vec<Arc<Mutex<MapChunk>>>>,
    pub biomes: Vec<Biome>,
    pub resources: Vec<ResourceDef>,
}

impl Map {
    pub fn new(
        width: u32,
        height: u32,
        biomes_path: &str,
        resources_path: &str,
    ) -> Result<Self, SimulationError> {
        let biomes_data = fs::read_to_string(biomes_path)?;
        let biomes: Vec<Biome> = serde_json::from_str(&biomes_data)?;

        let resources_data = fs::read_to_string(resources_path)?;
        let resources: Vec<ResourceDef> = serde_json::from_str(&resources_data)?;

        let chunks_x = (width as f32 / CHUNK_SIZE as f32).ceil() as usize;
        let chunks_y = (height as f32 / CHUNK_SIZE as f32).ceil() as usize;

        let chunks = (0..chunks_y)
            .map(|_| {
                (0..chunks_x)
                    .map(|_| Arc::new(Mutex::new(MapChunk::new())))
                    .collect()
            })
            .collect();

        let map = Map {
            width,
            height,
            chunks,
            biomes,
            resources,
        };

        Ok(map)
    }

    pub fn width_in_chunks(&self) -> u32 {
        (self.width as f32 / CHUNK_SIZE as f32).ceil() as u32
    }

    pub fn height_in_chunks(&self) -> u32 {
        (self.height as f32 / CHUNK_SIZE as f32).ceil() as u32
    }

    pub fn get_chunk_index(&self, x: u32, y: u32) -> Option<(usize, usize)> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(((x / CHUNK_SIZE) as usize, (y / CHUNK_SIZE) as usize))
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<Tile> {
        let (chunk_x, chunk_y) = self.get_chunk_index(x, y)?;
        let chunk = self.chunks.get(chunk_y)?.get(chunk_x)?.lock().ok()?;
        let local_x = (x % CHUNK_SIZE) as usize;
        let local_y = (y % CHUNK_SIZE) as usize;
        chunk.tiles.get(local_y)?.get(local_x).cloned()
    }

    pub fn set_tile(&self, x: u32, y: u32, tile: Tile) -> Option<()> {
        let (chunk_x, chunk_y) = self.get_chunk_index(x, y)?;
        let mut chunk = self.chunks.get(chunk_y)?.get(chunk_x)?.lock().ok()?;
        let local_x = (x % CHUNK_SIZE) as usize;
        let local_y = (y % CHUNK_SIZE) as usize;
        if let Some(t) = chunk.tiles.get_mut(local_y)?.get_mut(local_x) {
            *t = tile;
        }
        Some(())
    }

    pub fn add_entity_to_spatial_map(&self, entity: Entity, x: u32, y: u32) -> Option<()> {
        let (chunk_x, chunk_y) = self.get_chunk_index(x, y)?;
        let mut chunk = self.chunks.get(chunk_y)?.get(chunk_x)?.lock().ok()?;
        let local_x = x % CHUNK_SIZE;
        let local_y = y % CHUNK_SIZE;
        chunk
            .spatial_map
            .entry((local_x, local_y))
            .or_default()
            .push(entity);
        Some(())
    }

    pub fn remove_entity_from_spatial_map(&self, entity: Entity, x: u32, y: u32) -> Option<()> {
        let (chunk_x, chunk_y) = self.get_chunk_index(x, y)?;
        let mut chunk = self.chunks.get(chunk_y)?.get(chunk_x)?.lock().ok()?;
        let local_x = x % CHUNK_SIZE;
        let local_y = y % CHUNK_SIZE;
        if let Some(entities) = chunk.spatial_map.get_mut(&(local_x, local_y)) {
            entities.retain(|&e| e != entity);
        }
        Some(())
    }

    pub fn get_entities_at(&self, x: u32, y: u32) -> Option<Vec<Entity>> {
        let (chunk_x, chunk_y) = self.get_chunk_index(x, y)?;
        let chunk = self.chunks.get(chunk_y)?.get(chunk_x)?.lock().ok()?;
        let local_x = x % CHUNK_SIZE;
        let local_y = y % CHUNK_SIZE;
        chunk.spatial_map.get(&(local_x, local_y)).cloned()
    }

    pub fn is_walkable(&self, x: u32, y: u32) -> bool {
        self.get_tile(x, y)
            .is_some_and(|tile| matches!(tile.tile_type, '.' | ',' | 'f'))
    }

}
