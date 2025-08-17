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

use game::Game;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut game = Game::new(
        "biomes.json",
        "resources.json",
        "items.json",
        "recipes.json",
    );
    game.run().await?;

    Ok(())
}
