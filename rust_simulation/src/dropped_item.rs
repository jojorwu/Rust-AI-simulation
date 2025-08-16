use crate::entity::Entity;
use crate::actions::Action;
use crate::game::Game;
use crate::errors::SimulationError;
use std::any::Any;

pub struct DroppedItem {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub item: String,
    pub quantity: u32,
}

impl Entity for DroppedItem {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_position(&self) -> (u32, u32) {
        (self.x, self.y)
    }

    fn get_health(&self) -> i32 {
        1 // Dropped items can't be destroyed
    }

    fn is_alive(&self) -> bool {
        true
    }

    fn update(&mut self, _game: &Game) -> Result<Option<Action>, SimulationError> {
        // Dropped items are passive
        Ok(None)
    }
}
