use rust_simulation::config::{EPISODES, MAX_STEPS_PER_EPISODE};
use rust_simulation::renderer::Renderer;
use rust_simulation::road_builder;
use rust_simulation::{errors::SimulationError, Game};
use std::env;

fn main() -> Result<(), SimulationError> {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--hard-wipe".to_string()) {
        println!("Wiping simulation state...");

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let root_dir = std::path::Path::new(manifest_dir);

        let models_path = root_dir.join("../models");
        if models_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&models_path) {
                eprintln!("Failed to remove models directory: {e}");
            } else {
                println!("Removed models directory.");
            }
        }

        let q_table_path = root_dir.join("../q_table.json");
        if q_table_path.exists() {
            if let Err(e) = std::fs::remove_file(&q_table_path) {
                eprintln!("Failed to remove q_table.json: {e}");
            } else {
                println!("Removed q_table.json.");
            }
        }

        let sim_log_path = root_dir.join("../simulation_output.log");
        if sim_log_path.exists() {
            if let Err(e) = std::fs::remove_file(&sim_log_path) {
                eprintln!("Failed to remove simulation_output.log: {e}");
            } else {
                println!("Removed simulation_output.log.");
            }
        }

        println!("Wipe complete.");
        return Ok(());
    }

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

    let renderer = Renderer::new();
    renderer.print_intro();

    for episode in 0..EPISODES {
        for _step in 0..MAX_STEPS_PER_EPISODE {
            game.tick()?;
            let world = game
                .world
                .lock()
                .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;
            renderer.render(&game, &world);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        if (episode + 1) % 200 == 0 {
            println!("Episode {}/{}", episode + 1, EPISODES);
        }
    }

    renderer.print_outro();

    Ok(())
}
