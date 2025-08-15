mod map;
mod player;
mod state;
mod brain;
mod game;
mod config;
mod recipes;

use game::Game;

fn main() {
    let mut game = Game::new();
    game.run();
}
