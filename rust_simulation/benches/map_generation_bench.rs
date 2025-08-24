use criterion::{criterion_group, criterion_main, Criterion};
use rust_simulation::map_generator::generate_island_map;
use rust_simulation::map::Biome;
use std::fs;

fn setup_biomes() -> Vec<Biome> {
    let biomes_data = fs::read_to_string("data/biomes.json").unwrap();
    serde_json::from_str(&biomes_data).unwrap()
}

fn benchmark_map_generation(c: &mut Criterion) {
    let biomes = setup_biomes();
    c.bench_function("map_generation_1000x1000", |b| {
        b.iter(|| {
            generate_island_map(1000, 1000, &biomes, None, 25.0, 5, 0.5, 2.0).unwrap();
        })
    });
}

criterion_group!(benches, benchmark_map_generation);
criterion_main!(benches);
