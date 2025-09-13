//! The main entry point for the Rust Simulation application.
//!
//! This binary is responsible for:
//! - Parsing command-line arguments.
//! - Setting up application paths for data storage.
//! - Loading configuration files.
//! - Initializing the Bevy application (`App`).
//! - Adding all necessary plugins, resources, and systems from the `rust_simulation` library.
//! - Running the Bevy application.

use bevy::app::AppExit;
use bevy::prelude::*;
use rust_simulation::config::Config;
use rust_simulation::road_builder;
use rust_simulation::road_manager::RoadManager;
use rust_simulation::state::AppState;
use rust_simulation::systems::monitoring::MonitoringPlugin;
use rust_simulation::systems::persistence::save_q_tables_on_exit;
use rust_simulation::ui::main_menu::MainMenuPlugin;
use rust_simulation::ui::settings::SettingsPlugin;
use rust_simulation::{add_simulation_systems, setup_simulation, AppPaths, DataPaths, SimulationSet};
use std::env;
use std::time::Duration;
use bevy::app::ScheduleRunnerPlugin;
use directories::ProjectDirs;
use clap::Parser;
use bevy::asset::AssetPlugin;
use bevy::core_pipeline::CorePipelinePlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::input::InputPlugin;
use bevy::pbr::PbrPlugin;
use bevy::render::RenderPlugin;
use bevy::sprite::SpritePlugin;
use bevy::text::TextPlugin;
use bevy::transform::TransformPlugin;
use bevy::ui::UiPlugin;

/// Command-line arguments for the simulation.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Wipe all saved data (q-tables, models) before starting.
    #[arg(long)]
    hard_wipe: bool,
}

fn main() {
    // --- Path Setup ---
    let app_paths = if let Some(proj_dirs) = ProjectDirs::from("com", "simulation", "rust_simulation") {
        let data_dir = proj_dirs.data_dir().to_path_buf();
        if !data_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&data_dir) {
                eprintln!("Failed to create application data directory at {}: {}", data_dir.display(), e);
                std::process::exit(1);
            }
        }
        AppPaths { data_dir }
    } else {
        eprintln!("Could not determine application data directory.");
        std::process::exit(1);
    };

    let cli = Cli::parse();

    // --- This section is for wiping saved data, keeping it from the original main.rs ---
    if cli.hard_wipe {
        println!("Wiping simulation state...");
        let q_table_path = app_paths.data_dir.join("q_tables.json");
        if q_table_path.exists() {
            if let Err(e) = std::fs::remove_file(&q_table_path) {
                eprintln!("Failed to remove q_table.json: {e}");
            }
        }
        println!("Wipe complete.");
        return;
    }

    // --- Load Config ---
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{manifest_dir}/data/config.toml");
    let config = match Config::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config file at {}: {}", config_path, e);
            std::process::exit(1);
        }
    };


    // --- Bevy App Setup ---
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::log::LogPlugin::default());
    app.add_plugins(TransformPlugin);
    app.add_plugins(DiagnosticsPlugin);
    app.add_plugins(InputPlugin);
    app.add_plugins(ScheduleRunnerPlugin::default());
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(RenderPlugin::default());
    app.add_plugins(CorePipelinePlugin);
    app.add_plugins(SpritePlugin);
    app.add_plugins(TextPlugin);
    app.add_plugins(UiPlugin);
    app.add_plugins(PbrPlugin::default());
    app.add_plugins(rust_simulation::graphics::GraphicsPlugin);
    app.add_plugins(MainMenuPlugin);
    app.add_plugins(SettingsPlugin);
    app.add_plugins(MonitoringPlugin);
    app.init_state::<AppState>();
    app.register_type::<Config>()
        .register_type::<rust_simulation::config::MapSettings>()
        .register_type::<rust_simulation::config::PlayerSettings>()
        .register_type::<rust_simulation::config::TrainingSettings>()
        .register_type::<rust_simulation::config::DayNightCycle>()
        .register_type::<rust_simulation::config::Ai>()
        .register_type::<rust_simulation::config::QLearning>();
    app.register_type::<rust_simulation::config::Goals>();

    // --- Simulation Setup ---
    // Insert resources
    app.insert_resource(config);
    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(100)));
    app.init_resource::<RoadManager>();

    // Insert data paths resource
    app.insert_resource(app_paths);
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

    app.add_systems(Update, save_q_tables_on_exit.run_if(on_event::<AppExit>()));

    app.run();
}
