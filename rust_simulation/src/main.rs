use bevy::prelude::*;
use rust_simulation::config::Config;
use rust_simulation::errors::SimulationError;
use rust_simulation::road_builder;
use rust_simulation::road_manager::RoadManager;
use rust_simulation::state::AppState;
use rust_simulation::ui::main_menu::MainMenuPlugin;
use rust_simulation::ui::settings::SettingsPlugin;
use rust_simulation::{add_simulation_systems, setup_simulation, DataPaths, SimulationSet};
use std::env;
use std::time::Duration;

fn main() -> Result<(), SimulationError> {
    let args: Vec<String> = env::args().collect();

    // --- This section is for wiping saved data, keeping it from the original main.rs ---
    if args.contains(&"--hard-wipe".to_string()) {
        println!("Wiping simulation state...");
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let root_dir = std::path::Path::new(manifest_dir);
        let models_path = root_dir.join("../models");
        if models_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&models_path) {
                eprintln!("Failed to remove models directory: {e}");
            }
        }
        let q_table_path = root_dir.join("../q_table.json");
        if q_table_path.exists() {
            if let Err(e) = std::fs::remove_file(&q_table_path) {
                eprintln!("Failed to remove q_table.json: {e}");
            }
        }
        println!("Wipe complete.");
        return Ok(());
    }

    // --- Load Config ---
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{manifest_dir}/data/config.toml");
    let config = Config::load(&config_path)?;

    // --- Bevy App Setup ---
    let mut app = App::new();

    // Add default plugins and our custom graphics plugin
    app.add_plugins((MinimalPlugins, MainMenuPlugin, SettingsPlugin));
    app.init_state::<AppState>();
    app.register_type::<Config>()
        .register_type::<rust_simulation::config::MapSettings>()
        .register_type::<rust_simulation::config::PlayerSettings>()
        .register_type::<rust_simulation::config::TrainingSettings>()
        .register_type::<rust_simulation::config::DayNightCycle>()
        .register_type::<rust_simulation::config::Ai>()
        .register_type::<rust_simulation::config::QLearning>()
        .register_type::<rust_simulation::config::Goals>();

    // --- Simulation Setup ---
    // Insert resources
    app.insert_resource(config);
    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(100)));
    app.init_resource::<RoadManager>();

    // Insert data paths resource
    app.insert_resource(DataPaths {
        biomes: format!("{manifest_dir}/data/biomes.json"),
        resources: format!("{manifest_dir}/data/resources.json"),
        items: format!("{manifest_dir}/data/items.json"),
        recipes: format!("{manifest_dir}/data/recipes.json"),
    });

    // Add simulation setup and systems
    app.add_systems(
        OnEnter(AppState::InGame),
        (setup_simulation, road_builder::generate_roads)
            .chain()
            .in_set(SimulationSet::Setup),
    );

    add_simulation_systems(&mut app);

    app.run();

    Ok(())
}
