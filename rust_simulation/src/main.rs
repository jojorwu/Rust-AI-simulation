use bevy::app::AppExit;
use bevy::prelude::*;
use rust_simulation::config::Config;
use rust_simulation::errors::SimulationError;
use rust_simulation::road_builder;
use rust_simulation::road_manager::RoadManager;
use rust_simulation::state::AppState;
use rust_simulation::systems::monitoring::MonitoringPlugin;
use rust_simulation::systems::persistence::save_q_tables_on_exit;
use rust_simulation::ui::main_menu::MainMenuPlugin;
use rust_simulation::ui::settings::SettingsPlugin;
use rust_simulation::{add_simulation_systems, setup_simulation, DataPaths, SimulationSet};
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use bevy::app::ScheduleRunnerPlugin;
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

// Helper function to get the application's root directory.
// It checks for a cargo environment and falls back to the executable's directory.
fn get_app_root() -> PathBuf {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        if let Ok(mut path) = std::env::current_exe() {
            path.pop();
            path
        } else {
            PathBuf::from(".")
        }
    }
}

fn main() -> Result<(), SimulationError> {
    let args: Vec<String> = env::args().collect();
    let app_root = get_app_root();

    // --- This section is for wiping saved data ---
    if args.contains(&"--hard-wipe".to_string()) {
        println!("Wiping simulation state...");
        // For a distributable build, we assume models are in a subfolder.
        // For `cargo run`, this will be relative to the crate root.
        let models_path = if std::env::var("CARGO_MANIFEST_DIR").is_ok() {
             app_root.join("../models")
        } else {
            app_root.join("models")
        };

        if models_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&models_path) {
                eprintln!("Failed to remove models directory: {e}");
            }
        }

        let q_table_path = if std::env::var("CARGO_MANIFEST_DIR").is_ok() {
            app_root.join("../q_table.json")
        } else {
            app_root.join("q_table.json")
        };

        if q_table_path.exists() {
            if let Err(e) = std::fs::remove_file(&q_table_path) {
                eprintln!("Failed to remove q_table.json: {e}");
            }
        }
        println!("Wipe complete.");
        return Ok(());
    }

    // --- Load Config ---
    let config_path = app_root.join("data/config.toml");
    let mut config = Config::load(config_path.to_str().expect("Path is not valid UTF-8"))?;

    // --- Validate and Clamp Config ---
    let num_cpus = num_cpus::get();
    if config.performance.processor_cores as usize > num_cpus {
        log::warn!(
            "Processor cores setting ({}) is higher than the number of available cores ({}). Clamping to {}.",
            config.performance.processor_cores,
            num_cpus,
            num_cpus
        );
        config.performance.processor_cores = num_cpus as u32;
    }

    // --- Setup Rayon Thread Pool ---
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.performance.processor_cores as usize)
        .build_global()
        .unwrap();

    // --- Bevy App Setup ---
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::log::LogPlugin::default());
    app.add_plugins(TransformPlugin::default());
    app.add_plugins(DiagnosticsPlugin::default());
    app.add_plugins(InputPlugin::default());
    app.add_plugins(ScheduleRunnerPlugin::default());
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(RenderPlugin::default());
    app.add_plugins(CorePipelinePlugin::default());
    app.add_plugins(SpritePlugin::default());
    app.add_plugins(TextPlugin::default());
    app.add_plugins(UiPlugin::default());
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
        .register_type::<rust_simulation::config::QLearning>()
        .register_type::<rust_simulation::config::Goals>()
        .register_type::<rust_simulation::config::PerformanceSettings>();

    // --- Simulation Setup ---
    // Insert resources
    app.insert_resource(config);
    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(100)));
    app.init_resource::<RoadManager>();

    // Insert data paths resource
    let data_root = app_root.join("data");
    app.insert_resource(DataPaths {
        biomes: data_root.join("biomes.json").to_str().unwrap().to_string(),
        resources: data_root.join("resources.json").to_str().unwrap().to_string(),
        items: data_root.join("items.json").to_str().unwrap().to_string(),
        recipes: data_root.join("recipes.json").to_str().unwrap().to_string(),
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

    Ok(())
}
