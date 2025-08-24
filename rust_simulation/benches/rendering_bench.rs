use criterion::{criterion_group, criterion_main, Criterion};
use rust_simulation::graphics::rendering::map_rendering::setup_map_meshes;
use bevy::prelude::*;
use rust_simulation::config::Config;
use rust_simulation::map::Map;
use rust_simulation::player::Player;

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let config = Config::load("data/config.toml").unwrap();
    let map = Map::new(
        config.map_settings.width,
        config.map_settings.height,
        "data/biomes.json",
        "data/resources.json",
        config.map_settings.seed,
    )
    .unwrap();
    app.insert_resource(map);
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    app.world.spawn(Player::new(0, 100, 100));

    app
}

fn benchmark_setup_chunk_meshes(c: &mut Criterion) {
    c.bench_function("setup_chunk_meshes", |b| {
        b.iter(|| {
            let mut app = setup_app();
            app.add_systems(Startup, setup_map_meshes);
            app.update();
        })
    });
}

criterion_group!(benches, benchmark_setup_chunk_meshes);
criterion_main!(benches);
