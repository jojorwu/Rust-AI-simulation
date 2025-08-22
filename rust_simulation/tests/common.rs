use bevy_ecs::{prelude::*, schedule::Schedule};
use rust_simulation::errors::SimulationError;
use rust_simulation::setup_world;
use std::env;
use std::time::Duration;

pub fn create_test_world() -> Result<World, SimulationError> {
    let _ = env_logger::try_init();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|e| SimulationError::EnvVarError(e.to_string()))?;
    setup_world(
        &format!("{manifest_dir}/data/biomes.json"),
        &format!("{manifest_dir}/data/resources.json"),
        &format!("{manifest_dir}/data/items.json"),
        &format!("{manifest_dir}/data/recipes.json"),
    )
}

/// Runs a schedule in a loop until a condition is met or a timeout occurs.
pub fn run_schedule_until<F>(
    world: &mut World,
    mut schedule: Schedule,
    condition: F,
    timeout_ticks: u32,
) -> bool
where
    F: Fn(&mut World) -> bool,
{
    let mut condition_met = false;
    for _ in 0..timeout_ticks {
        schedule.run(world);
        if condition(world) {
            condition_met = true;
            break;
        }
        // Sleep to allow async tasks to complete
        std::thread::sleep(Duration::from_millis(10));
    }
    condition_met
}
