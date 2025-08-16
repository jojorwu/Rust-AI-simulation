use crate::actions::Action;
use crate::game::Game;
use crate::errors::SimulationError;
use std::any::Any;

pub trait Entity {
    fn as_any(&mut self) -> &mut dyn Any;
    fn get_id(&self) -> u32;
    fn get_position(&self) -> (u32, u32);
    fn get_health(&self) -> i32;
    fn is_alive(&self) -> bool;
    fn update(&mut self, game: &Game) -> Result<Option<Action>, SimulationError>;
}
