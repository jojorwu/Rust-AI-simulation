use crate::config::Config;
use crate::errors::SimulationError;
use crate::events::Event;
use crate::map::{Biome, Map, Tile, CHUNK_SIZE};
use bevy_ecs::prelude::*;
use noise::{Fbm, NoiseFn, OpenSimplex, RidgedMulti};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::sync::Arc;

pub fn trigger_map_generation_system(
    mut event_writer: EventWriter<Event>,
    map: Res<Map>,
    config: Res<Config>,
) {
    let generated_chunks = generate_island_map(
        map.width,
        map.height,
        &map.biomes,
        config.map_settings.seed,
        25.0,
        5,
        0.5,
        2.0,
    )
    .unwrap();

    for (position, tiles) in generated_chunks {
        event_writer.send(Event::ChunkGenerated { position, tiles });
    }
}

pub fn generate_island_map(
    width: u32,
    height: u32,
    biomes: &[Biome],
    seed: Option<u32>,
    scale: f64,
    _octaves: i32,
    _persistence: f64,
    _lacunarity: f64,
) -> Result<Vec<((u32, u32), Vec<Vec<Tile>>)>, SimulationError> {
    let seed_val = seed.unwrap_or_else(rand::random);
    let mut rng = StdRng::seed_from_u64(seed_val as u64);

    let mut random_numbers = vec![vec![0; width as usize]; height as usize];
    for y in 0..height {
        for x in 0..width {
            random_numbers[y as usize][x as usize] = rng.gen_range(0..100);
        }
    }
    let random_numbers = Arc::new(random_numbers);

    let base_fbm: Arc<Fbm<OpenSimplex>> = Arc::new(Fbm::new(seed_val));
    let ridged_multi: Arc<RidgedMulti<OpenSimplex>> =
        Arc::new(RidgedMulti::new(seed_val.wrapping_add(1)));

    let width_in_chunks = (width as f32 / CHUNK_SIZE as f32).ceil() as u32;
    let height_in_chunks = (height as f32 / CHUNK_SIZE as f32).ceil() as u32;

    let generated_chunks: Vec<((u32, u32), Vec<Vec<Tile>>)> = (0..height_in_chunks)
        .into_par_iter()
        .flat_map(|chunk_y| {
            let base_fbm = base_fbm.clone();
            let ridged_multi = ridged_multi.clone();
            let random_numbers = random_numbers.clone();
            (0..width_in_chunks)
                .into_par_iter()
                .map(move |chunk_x| {
                    let mut tiles =
                        vec![
                            vec![Tile::new(' ', "none".to_string()); CHUNK_SIZE as usize];
                            CHUNK_SIZE as usize
                        ];
                    for y_local in 0..CHUNK_SIZE {
                        for x_local in 0..CHUNK_SIZE {
                            let x_abs = chunk_x * CHUNK_SIZE + x_local;
                            let y_abs = chunk_y * CHUNK_SIZE + y_local;

                            if x_abs < width && y_abs < height {
                                let nx = 2.0 * x_abs as f64 / width as f64 - 1.0;
                                let ny = 2.0 * y_abs as f64 / height as f64 - 1.0;
                                let dist = 1.0 - (1.0 - nx.powi(2)) * (1.0 - ny.powi(2));

                                let pos = [x_abs as f64 / scale, y_abs as f64 / scale];
                                let base_noise = base_fbm.get(pos);
                                let mountain_pos =
                                    [x_abs as f64 / (scale * 2.5), y_abs as f64 / (scale * 2.5)];
                                let mountain_noise = ridged_multi.get(mountain_pos);
                                let combined_noise = base_noise + (mountain_noise.powi(2) * 0.6);
                                let island_val = (combined_noise.clamp(-1.0, 1.5) + 1.0) / 2.5;
                                let height_val = island_val * (1.0 - dist);

                                let mut tile_char = ' ';
                                let mut biome_name = "none".to_string();

                                for biome in biomes {
                                    if height_val >= biome.height_range[0]
                                        && height_val < biome.height_range[1]
                                    {
                                        tile_char = biome.tile_type;
                                        biome_name = biome.name.clone();
                                        break;
                                    }
                                }

                                if biome_name == "plains"
                                    && tile_char == '.'
                                    && random_numbers[y_abs as usize][x_abs as usize] < 5
                                {
                                    tile_char = 'f';
                                }

                                tiles[y_local as usize][x_local as usize] =
                                    Tile::new(tile_char, biome_name);
                            }
                        }
                    }
                    ((chunk_x, chunk_y), tiles)
                })
        })
        .collect();

    Ok(generated_chunks)
}
