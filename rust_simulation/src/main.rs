mod map;
mod player;
mod state;
mod brain;
mod game;
mod config;
mod recipes;
mod errors;

use game::Game;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut game = Game::new();
    game.run()?;

    Ok(())
}
