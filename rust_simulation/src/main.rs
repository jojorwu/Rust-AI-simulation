mod map;
mod pathfinding;
mod player;
mod ecs;
mod components;
mod systems;
mod state;
mod brain;
mod game;
mod item;
mod config;
mod recipes;
mod errors;
mod events;
mod fov;

use game::Game;
use std::error::Error;
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut game = Game::new(
        "biomes.json",
        "resources.json",
        "items.json",
        "recipes.json",
    );

    if args.contains(&"--wipe".to_string()) {
        game.new_generation()?;
    }

    game.run()?;

    Ok(())
}
