use noise::{NoiseFn, Fbm, Perlin};
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::fs;
use std::error::Error;
use super::player::Player;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tile {
    pub tile_type: char,
    pub biome: String,
    pub lock_id: Option<u32>,
    pub remaining_resources: Option<u32>,
    pub depletion_episode: Option<u32>,
    pub original_tile_type: char,
    pub health: Option<f64>,
}

impl Tile {
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

#[derive(Debug, Deserialize)]
pub struct Biome {
    pub name: String,
    pub tile_type: char,
    pub height_range: [f64; 2],
}

#[derive(Debug, Deserialize)]
pub struct Resource {
    pub name: String,
    pub tile_type: char,
    pub biomes: Vec<String>,
    pub density: f64,
}

pub struct Map {
    pub width: u32,
    pub height: u32,
    pub grid: Vec<Vec<Tile>>,
    pub biomes: Vec<Biome>,
    pub resources: Vec<Resource>,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let biomes_data = fs::read_to_string("biomes.json")?;
        let biomes: Vec<Biome> = serde_json::from_str(&biomes_data)?;

        let resources_data = fs::read_to_string("resources.json")?;
        let resources: Vec<Resource> = serde_json::from_str(&resources_data)?;

        let grid = vec![vec![Tile::new(' ', "none".to_string()); width as usize]; height as usize];

        Ok(Map {
            width,
            height,
            grid,
            biomes,
            resources,
        })
    }

    pub fn generate_island_map(&mut self, scale: f64, octaves: i32, persistence: f64, lacunarity: f64) {
        let seed = rand::thread_rng().r#gen::<u32>();
        let mut fbm: Fbm<Perlin> = Fbm::new(seed);
        fbm.octaves = octaves as usize;
        fbm.persistence = persistence;
        fbm.lacunarity = lacunarity;

        for y in 0..self.height {
            for x in 0..self.width {
                let nx = 2.0 * x as f64 / self.width as f64 - 1.0;
                let ny = 2.0 * y as f64 / self.height as f64 - 1.0;
                let dist = 1.0 - (1.0 - nx.powi(2)) * (1.0 - ny.powi(2));
                let noise_val = fbm.get([x as f64 / scale, y as f64 / scale]);
                let island_val = (noise_val + 1.0) / 2.0;
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
                self.grid[y as usize][x as usize] = Tile::new(tile_char, biome_name);
            }
        }
    }

    pub fn display(&self, world: &super::ecs::World) {
        for y in 0..self.height {
            for x in 0..self.width {
                let mut entity_on_tile = None;
                for entity in 0..world.entities.len() {
                    if let Some(pos) = world.get_component::<super::components::Position>(entity) {
                        if pos.x == x && pos.y == y {
                            entity_on_tile = Some(entity);
                            break;
                        }
                    }
                }

                if let Some(entity) = entity_on_tile {
                    if world.get_component::<Player>(entity).is_some() {
                        print!("P ");
                    } else {
                        print!("E ");
                    }
                } else {
                    print!("{} ", self.grid[y as usize][x as usize].tile_type);
                }
            }
            println!();
        }
    }

    pub fn add_resource(&mut self, x: u32, y: u32, resource_type: char) {
        let tile = &mut self.grid[y as usize][x as usize];
        tile.tile_type = resource_type;
        tile.original_tile_type = resource_type;
        tile.remaining_resources = Some(5);
    }
}
