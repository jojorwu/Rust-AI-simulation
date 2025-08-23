use bevy::prelude::*;
use rust_simulation::graphics::GraphicsPlugin;
use rust_simulation::road_builder;
use rust_simulation::{errors::SimulationError, Game};
use std::env;
use std::time::Duration;

// Define a simple system to run the game's tick function
fn game_tick_system(mut game: ResMut<Game>) {
    if let Err(e) = game.tick() {
        eprintln!("Error during game tick: {}", e);
    }
}

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

    // --- Game Initialization ---
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut game = Game::new(
        &format!("{manifest_dir}/data/biomes.json"),
        &format!("{manifest_dir}/data/resources.json"),
        &format!("{manifest_dir}/data/items.json"),
        &format!("{manifest_dir}/data/recipes.json"),
    )?;

    road_builder::generate_roads(&mut game)?;

    if args.contains(&"--wipe".to_string()) {
        game.new_generation()?;
    }

    // --- Bevy App Setup ---
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, GraphicsPlugin))
        .insert_resource(game)
        // Run the game_tick_system on a fixed schedule
        .add_systems(FixedUpdate, game_tick_system)
        // Configure the fixed timestep
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(100)));

    app.run();

    Ok(())
}
