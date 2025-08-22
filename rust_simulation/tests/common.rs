use bevy_ecs::prelude::World;
use rust_simulation::errors::SimulationError;
use rust_simulation::setup_world;
use std::env;

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
