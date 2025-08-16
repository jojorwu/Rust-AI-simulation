use noise::{NoiseFn, Fbm, Perlin};
use rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tile {
    pub tile_type: char,
    pub lock_id: Option<u32>,
    pub remaining_resources: Option<u32>,
    pub depletion_episode: Option<u32>,
    pub original_tile_type: char,
}

impl Tile {
    pub fn new(tile_type: char) -> Self {
        Tile {
            tile_type,
            lock_id: None,
            remaining_resources: None,
            depletion_episode: None,
            original_tile_type: tile_type,
        }
    }
}

pub struct Map {
    pub width: u32,
    pub height: u32,
    pub grid: Vec<Vec<Tile>>,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        Map {
            width,
            height,
            grid: vec![vec![Tile::new(' '); width as usize]; height as usize],
        }
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

                let tile_char = if height < 0.1 { 'W' }
                else if height < 0.15 { 'S' }
                else if height < 0.5 { '.' }
                else { 'M' };
                self.grid[y as usize][x as usize] = Tile::new(tile_char);
            }
        }
    }

    pub fn display(&self, players: &[super::player::Player]) {
        for y in 0..self.height {
            for x in 0..self.width {
                let mut is_player_on_tile = false;
                for p in players {
                    if p.x == x && p.y == y {
                        print!("P ");
                        is_player_on_tile = true;
                        break;
                    }
                }
                if !is_player_on_tile {
                    print!("{} ", self.grid[y as usize][x as usize].tile_type);
                }
            }
            println!();
        }
    }

    pub fn add_tree(&mut self, x: u32, y: u32) {
        let tile = &mut self.grid[y as usize][x as usize];
        tile.tile_type = 'T';
        tile.original_tile_type = 'T';
        tile.remaining_resources = Some(5);
    }

    pub fn add_rock(&mut self, x: u32, y: u32) {
        let tile = &mut self.grid[y as usize][x as usize];
        tile.tile_type = 'R';
        tile.original_tile_type = 'R';
        tile.remaining_resources = Some(5);
    }

    pub fn add_sulfur(&mut self, x: u32, y: u32) {
        let tile = &mut self.grid[y as usize][x as usize];
        tile.tile_type = 'U';
        tile.original_tile_type = 'U';
        tile.remaining_resources = Some(5);
    }

    pub fn add_iron_ore_node(&mut self, x: u32, y: u32) {
        let tile = &mut self.grid[y as usize][x as usize];
        tile.tile_type = 'I';
        tile.original_tile_type = 'I';
        tile.remaining_resources = Some(5);
    }
}
