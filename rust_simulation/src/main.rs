mod actions;
mod map;
mod pathfinding;
mod player;
mod state;
mod brain;
mod game;
mod item;
mod config;
mod recipes;
mod errors;

use game::Game;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut game = Game::new();
    game.run().await?;

    Ok(())
}
