use noise::{NoiseFn, Fbm, Perlin};
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::fs;
use std::error::Error;
use super::player::Player;
use std::collections::HashMap;
use super::ecs::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TileState {
    Unseen,
    Explored,
    Visible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalMap {
    pub width: u32,
    pub height: u32,
    pub grid: Vec<Vec<TileState>>,
}

impl MentalMap {
    pub fn new(width: u32, height: u32) -> Self {
        MentalMap {
            width,
            height,
            grid: vec![vec![TileState::Unseen; width as usize]; height as usize],
        }
    }
}

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

#[derive(Debug, Deserialize, Clone)]
pub struct Resource {
    pub name: String,
    pub biomes: Vec<String>,
    pub density: f64,
}

pub struct Map {
    pub width: u32,
    pub height: u32,
    pub grid: Vec<Vec<Tile>>,
    pub biomes: Vec<Biome>,
    pub resources: Vec<Resource>,
    pub spatial_map: HashMap<(u32, u32), Vec<Entity>>,
}

impl Map {
    pub fn display_observer_map(&self, world: &super::ecs::World) {
        println!("\n--- Observer Map ---");
        for y in 0..self.height {
            for x in 0..self.width {
                let entity_on_tile = self.spatial_map.get(&(x, y)).and_then(|v| v.first());

                if let Some(&entity) = entity_on_tile {
                    if world.get_component::<Player>(entity).is_some() {
                        print!("\x1b[91mP \x1b[0m"); // Bright Red 'P'
                    } else {
                        print!("\x1b[33mE \x1b[0m"); // Yellow 'E'
                    }
                } else {
                    let tile_char = self.grid[y as usize][x as usize].tile_type;
                    match tile_char {
                        '.' => print!("\x1b[32m. \x1b[0m"),   // Green
                        'f' => print!("\x1b[93mf \x1b[0m"),   // Bright Yellow
                        'M' => print!("\x1b[97mM \x1b[0m"),   // Bright White
                        'T' => print!("\x1b[32m T\x1b[0m"),  // Dark Green
                        '~' => print!("\x1b[34m~ \x1b[0m"),   // Blue
                        '#' => print!("\x1b[90m# \x1b[0m"),   // Dim White
                        'O' => print!("\x1b[36mO \x1b[0m"),   // Cyan
                        _ => print!("{} ", tile_char),
                    }
                }
            }
            println!();
        }
    }

    pub fn new(width: u32, height: u32, biomes_path: &str, resources_path: &str) -> Result<Self, Box<dyn Error>> {
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

                if biome_name == "plains" && tile_char == '.' && rand::thread_rng().gen_range(0..100) < 5 {
                    tile_char = 'f';
                }

                self.grid[y as usize][x as usize] = Tile::new(tile_char, biome_name);
            }
        }
    }

    pub fn display(&self, world: &super::ecs::World) {
        // For multi-player, we'd need to specify which player's map to show.
        // For now, we'll just find the first entity with a Player component.
        let player_entity = (0..world.entities.len()).find(|&e| world.get_component::<Player>(e).is_some());

        if let Some(player_entity) = player_entity {
            if let Some(player) = world.get_component::<Player>(player_entity) {
                let mental_map = &player.mental_map;

                for y in 0..self.height {
                    for x in 0..self.width {
                        let tile_state = mental_map.grid[y as usize][x as usize];
                        match tile_state {
                            TileState::Unseen => print!("  "), // Two spaces for alignment
                            TileState::Explored => {
                                print!("\x1b[90m{} \x1b[0m", self.grid[y as usize][x as usize].tile_type); // Dim gray color
                            }
                            TileState::Visible => {
                                let entity_on_tile = self.spatial_map.get(&(x, y)).and_then(|v| v.first());

                                if let Some(&entity) = entity_on_tile {
                                    if world.get_component::<Player>(entity).is_some() {
                                        print!("\x1b[91mP \x1b[0m"); // Bright Red 'P'
                                    } else {
                                        print!("\x1b[33mE \x1b[0m"); // Yellow 'E'
                                    }
                                } else {
                                    print!("\x1b[97m{} \x1b[0m", self.grid[y as usize][x as usize].tile_type); // Bright White
                                }
                            }
                        }
                    }
                    println!();
                }
            }
        } else {
            // Fallback if no player is found (e.g., during setup or for debugging)
            for y in 0..self.height {
                for x in 0..self.width {
                    print!("{} ", self.grid[y as usize][x as usize].tile_type);
                }
                println!();
            }
        }
    }
}
