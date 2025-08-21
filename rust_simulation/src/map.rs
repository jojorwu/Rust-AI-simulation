//! This module handles the generation and representation of the game world.
//!
//! It includes structures for the `Map` itself, individual `Tile`s, `Biome`s,
//! and `Resource`s. It also contains the logic for procedural island generation
//! using noise functions.

use super::ecs::Entity;
use noise::{Fbm, NoiseFn, OpenSimplex, RidgedMulti};
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::errors::SimulationError;
use std::collections::HashMap;
use std::fs;

/// Represents the visibility state of a tile from a player's perspective.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TileState {
    /// The tile has not yet been seen by the player.
    Unseen,
    /// The tile has been seen previously but is not currently in view.
    Explored,
    /// The tile is currently visible to the player.
    Visible,
}

/// Represents a player's memory of the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalMap {
    /// The width of the map.
    pub width: u32,
    /// The height of the map.
    pub height: u32,
    /// The grid of tile states representing the player's memory.
    pub grid: Vec<Vec<TileState>>,
}

impl MentalMap {
    /// Creates a new `MentalMap` of the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        MentalMap {
            width,
            height,
            grid: vec![vec![TileState::Unseen; width as usize]; height as usize],
        }
    }
}

/// Represents a single tile on the game map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tile {
    /// The character representation of the tile type (e.g., '.', 'T', '~').
    pub tile_type: char,
    /// The name of the biome this tile belongs to.
    pub biome: String,
    /// An optional ID of an entity that has locked this tile.
    pub lock_id: Option<u32>,
    /// The number of resources remaining on this tile, if any.
    pub remaining_resources: Option<u32>,
    /// The simulation episode when this tile's resources were depleted.
    pub depletion_episode: Option<u32>,
    /// The original tile type before any changes (e.g., resource depletion).
    pub original_tile_type: char,
    /// The health of a structure on this tile, if any.
    pub health: Option<f64>,
}

impl Tile {
    /// Creates a new `Tile`.
    pub fn new(tile_type: char, biome: String) -> Self {
        Tile {
            tile_type,
            biome,
            lock_id: None,
            remaining_resources: None,
            depletion_episode: None,
            original_tile_type: tile_type,
            health: None,
        }
    }
}

/// Represents a biome type with its associated properties.
#[derive(Debug, Deserialize)]
pub struct Biome {
    /// The name of the biome.
    pub name: String,
    /// The character representation of the biome's default tile.
    pub tile_type: char,
    /// The height range [min, max] for this biome to generate.
    pub height_range: [f64; 2],
}

/// Represents a resource that can be found in the world.
#[derive(Debug, Deserialize, Clone)]
pub struct Resource {
    /// The name of the resource.
    pub name: String,
    /// A list of biomes where this resource can be found.
    pub biomes: Vec<String>,
    /// The density of this resource within its biomes.
    pub density: f64,
}

/// Represents the game map, containing the grid of tiles and other world data.
pub struct Map {
    /// The width of the map in tiles.
    pub width: u32,
    /// The height of the map in tiles.
    pub height: u32,
    /// The 2D grid of tiles that make up the map.
    pub grid: Vec<Vec<Tile>>,
    /// The list of biome definitions.
    pub biomes: Vec<Biome>,
    /// The list of resource definitions.
    pub resources: Vec<Resource>,
    /// A spatial hash map to quickly look up entities at a given position.
    pub spatial_map: HashMap<(u32, u32), Vec<Entity>>,
}

impl Map {
    /// Creates a new `Map` instance from configuration files.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the map to create.
    /// * `height` - The height of the map to create.
    /// * `biomes_path` - The path to the biomes JSON configuration file.
    /// * `resources_path` - The path to the resources JSON configuration file.
    pub fn new(
        width: u32,
        height: u32,
        biomes_path: &str,
        resources_path: &str,
    ) -> Result<Self, SimulationError> {
        let biomes_data = fs::read_to_string(biomes_path)?;
        let biomes: Vec<Biome> = serde_json::from_str(&biomes_data)?;

        let resources_data = fs::read_to_string(resources_path)?;
        let resources: Vec<Resource> = serde_json::from_str(&resources_data)?;

        let grid = vec![vec![Tile::new(' ', "none".to_string()); width as usize]; height as usize];

        Ok(Map {
            width,
            height,
            grid,
            biomes,
            resources,
            spatial_map: HashMap::new(),
        })
    }

    /// Generates a procedural island map using noise functions.
    ///
    /// This method uses a combination of FBM (Fractional Brownian Motion) and
    /// RidgedMulti noise to create a base terrain, then applies an island mask
    /// to shape it into an island.
    ///
    /// # Arguments
    ///
    /// * `scale` - The scale of the noise function (a larger value means more zoomed in).
    /// * `octaves` - The number of octaves for the FBM noise.
    /// * `persistence` - The persistence for the FBM noise.
    /// * `lacunarity` - The lacunarity for the FBM noise.
    pub fn generate_island_map(
        &mut self,
        scale: f64,
        octaves: i32,
        persistence: f64,
        lacunarity: f64,
    ) {
        let seed = rand::rng().random::<u32>();

        // Base terrain using OpenSimplex
        let mut base_fbm: Fbm<OpenSimplex> = Fbm::new(seed);
        base_fbm.octaves = octaves as usize;
        base_fbm.persistence = persistence;
        base_fbm.lacunarity = lacunarity;

        // Mountainous terrain using RidgedMulti
        let ridged_multi: RidgedMulti<OpenSimplex> = RidgedMulti::new(seed.wrapping_add(1));
        // We can tune RidgedMulti properties here if needed, e.g., frequency, octaves.
        // For now, we'll use defaults and a different scale.

        for y in 0..self.height {
            for x in 0..self.width {
                // Island mask calculations
                let nx = 2.0 * x as f64 / self.width as f64 - 1.0;
                let ny = 2.0 * y as f64 / self.height as f64 - 1.0;
                let dist = 1.0 - (1.0 - nx.powi(2)) * (1.0 - ny.powi(2));

                // Noise coordinates
                let pos = [x as f64 / scale, y as f64 / scale];

                // Calculate base noise
                let base_noise = base_fbm.get(pos);

                // Calculate mountain noise at a different scale (larger features)
                let mountain_pos = [x as f64 / (scale * 2.5), y as f64 / (scale * 2.5)];
                let mountain_noise = ridged_multi.get(mountain_pos);

                // Combine the noise values.
                // The base noise provides the general elevation.
                // The mountain_noise is weighted and added to create peaks and valleys.
                // We square the mountain_noise to make the ridges sharper.
                let combined_noise = base_noise + (mountain_noise.powi(2) * 0.6);

                // Normalize and apply island mask
                let island_val = (combined_noise.clamp(-1.0, 1.5) + 1.0) / 2.5; // Adjust clamping and normalization for the new range
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

                // Keep the random flower generation
                if biome_name == "plains"
                    && tile_char == '.'
                    && rand::rng().random_range(0..100) < 5
                {
                    tile_char = 'f';
                }

                self.grid[y as usize][x as usize] = Tile::new(tile_char, biome_name);
            }
        }
    }

}
