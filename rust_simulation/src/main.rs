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

fn main() -> Result<(), Box<dyn Error>> {
    let mut game = Game::new(
        "biomes.json",
        "resources.json",
        "items.json",
        "recipes.json",
    );
    game.run()?;

    Ok(())
}
