//! This module handles the representation of the game world, divided into chunks for concurrent access.

use bevy_ecs::prelude::*;
use noise::{Fbm, NoiseFn, OpenSimplex, RidgedMulti};
use rand::Rng;
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

        let mut map = Map {
            width,
            height,
            chunks,
            biomes,
            resources,
        };

        map.generate_island_map(25.0, 5, 0.5, 2.0);
        Ok(map)
    }

    pub fn get_chunk_index(&self, x: u32, y: u32) -> Option<(usize, usize)> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(((x / CHUNK_SIZE) as usize, (y / CHUNK_SIZE) as usize))
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<Tile> {
        let (chunk_x, chunk_y) = self.get_chunk_index(x, y)?;
        let chunk = self.chunks.get(chunk_y)?.get(chunk_x)?.lock().unwrap();
        let local_x = (x % CHUNK_SIZE) as usize;
        let local_y = (y % CHUNK_SIZE) as usize;
        chunk.tiles.get(local_y)?.get(local_x).cloned()
    }

    pub fn set_tile(&self, x: u32, y: u32, tile: Tile) -> Option<()> {
        let (chunk_x, chunk_y) = self.get_chunk_index(x, y)?;
        let mut chunk = self.chunks.get(chunk_y)?.get(chunk_x)?.lock().unwrap();
        let local_x = (x % CHUNK_SIZE) as usize;
        let local_y = (y % CHUNK_SIZE) as usize;
        if let Some(t) = chunk.tiles.get_mut(local_y)?.get_mut(local_x) {
            *t = tile;
        }
        Some(())
    }

    pub fn add_entity_to_spatial_map(&self, entity: Entity, x: u32, y: u32) -> Option<()> {
        let (chunk_x, chunk_y) = self.get_chunk_index(x, y)?;
        let mut chunk = self.chunks.get(chunk_y)?.get(chunk_x)?.lock().unwrap();
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
        let mut chunk = self.chunks.get(chunk_y)?.get(chunk_x)?.lock().unwrap();
        let local_x = x % CHUNK_SIZE;
        let local_y = y % CHUNK_SIZE;
        if let Some(entities) = chunk.spatial_map.get_mut(&(local_x, local_y)) {
            entities.retain(|&e| e != entity);
        }
        Some(())
    }

    pub fn get_entities_at(&self, x: u32, y: u32) -> Option<Vec<Entity>> {
        let (chunk_x, chunk_y) = self.get_chunk_index(x, y)?;
        let chunk = self.chunks.get(chunk_y)?.get(chunk_x)?.lock().unwrap();
        let local_x = x % CHUNK_SIZE;
        let local_y = y % CHUNK_SIZE;
        chunk.spatial_map.get(&(local_x, local_y)).cloned()
    }

    pub fn is_walkable(&self, x: u32, y: u32) -> bool {
        self.get_tile(x, y)
            .map_or(false, |tile| matches!(tile.tile_type, '.' | ',' | 'f'))
    }

    fn generate_island_map(&mut self, scale: f64, octaves: i32, persistence: f64, lacunarity: f64) {
        let mut rng = rand::rng();
        let seed = rng.random::<u32>();

        let mut base_fbm: Fbm<OpenSimplex> = Fbm::new(seed);
        base_fbm.octaves = octaves as usize;
        base_fbm.persistence = persistence;
        base_fbm.lacunarity = lacunarity;

        let ridged_multi: RidgedMulti<OpenSimplex> = RidgedMulti::new(seed.wrapping_add(1));

        for y_abs in 0..self.height {
            for x_abs in 0..self.width {
                let nx = 2.0 * x_abs as f64 / self.width as f64 - 1.0;
                let ny = 2.0 * y_abs as f64 / self.height as f64 - 1.0;
                let dist = 1.0 - (1.0 - nx.powi(2)) * (1.0 - ny.powi(2));

                let pos = [x_abs as f64 / scale, y_abs as f64 / scale];
                let base_noise = base_fbm.get(pos);
                let mountain_pos = [x_abs as f64 / (scale * 2.5), y_abs as f64 / (scale * 2.5)];
                let mountain_noise = ridged_multi.get(mountain_pos);
                let combined_noise = base_noise + (mountain_noise.powi(2) * 0.6);
                let island_val = (combined_noise.clamp(-1.0, 1.5) + 1.0) / 2.5;
                let height = island_val * (1.0 - dist);

                let mut tile_char = ' ';
                let mut biome_name = "none".to_string();

                for biome in &self.biomes {
                    if height >= biome.height_range[0] && height < biome.height_range[1] {
                        tile_char = biome.tile_type;
                        biome_name = biome.name.clone();
                        break;
                    }
                }

                if biome_name == "plains" && tile_char == '.' && rng.random_range(0..100) < 5 {
                    tile_char = 'f';
                }

                self.set_tile(x_abs, y_abs, Tile::new(tile_char, biome_name));
            }
        }
    }
}
