use noise::{NoiseFn, Fbm};
use rand::Rng;

pub struct Map {
    pub width: u32,
    pub height: u32,
    pub grid: Vec<Vec<char>>,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        Map {
            width,
            height,
            grid: vec![vec![' '; width as usize]; height as usize],
        }
    }

    pub fn generate_island_map(&mut self, scale: f64, octaves: i32, persistence: f64, lacunarity: f64) {
        let seed = rand::thread_rng().gen::<u32>();
        let mut fbm = Fbm::new(seed);
        fbm.octaves = octaves;
        fbm.persistence = persistence;
        fbm.lacunarity = lacunarity;

        for y in 0..self.height {
            for x in 0..self.width {
                // Calculate distance from center for radial gradient
                let nx = 2.0 * x as f64 / self.width as f64 - 1.0;
                let ny = 2.0 * y as f64 / self.height as f64 - 1.0;
                let dist = 1.0 - (1.0 - nx.powi(2)) * (1.0 - ny.powi(2));

                // Generate FBM noise value
                let noise_val = fbm.get([x as f64 / scale, y as f64 / scale]);

                // Combine noise with radial gradient to form an island
                let island_val = (noise_val + 1.0) / 2.0; // Normalize to 0-1
                let height = island_val * (1.0 - dist);

                // Assign tile based on height
                let tile = if height < 0.1 {
                    'W' // Water
                } else if height < 0.15 {
                    'S' // Sand
                } else if height < 0.5 {
                    '.' // Plains
                } else {
                    'M' // Mountain
                };
                self.grid[y as usize][x as usize] = tile;
            }
        }
    }

    pub fn display(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                print!("{} ", self.grid[y as usize][x as usize]);
            }
            println!();
        }
    }
}
