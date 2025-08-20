use rust_simulation::Game;
use rust_simulation::road_builder;
use std::error::Error;
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--hard-wipe".to_string()) {
        println!("Wiping simulation state...");

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let root_dir = std::path::Path::new(manifest_dir);

        let models_path = root_dir.join("../models");
        if models_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&models_path) {
                eprintln!("Failed to remove models directory: {}", e);
            } else {
                println!("Removed models directory.");
            }
        }

        let q_table_path = root_dir.join("../q_table.json");
        if q_table_path.exists() {
            if let Err(e) = std::fs::remove_file(&q_table_path) {
                eprintln!("Failed to remove q_table.json: {}", e);
            } else {
                println!("Removed q_table.json.");
            }
        }

        let sim_log_path = root_dir.join("../simulation_output.log");
        if sim_log_path.exists() {
            if let Err(e) = std::fs::remove_file(&sim_log_path) {
                eprintln!("Failed to remove simulation_output.log: {}", e);
            } else {
                println!("Removed simulation_output.log.");
            }
        }

        println!("Wipe complete.");
        return Ok(());
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut game = Game::new(
        &format!("{}/data/biomes.json", manifest_dir),
        &format!("{}/data/resources.json", manifest_dir),
        &format!("{}/data/items.json", manifest_dir),
        &format!("{}/data/recipes.json", manifest_dir),
    );

    road_builder::generate_roads(&mut game)?;

    if args.contains(&"--wipe".to_string()) {
        game.new_generation()?;
    }

    game.run()?;

    Ok(())
}
