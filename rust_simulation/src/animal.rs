use crate::entity::Entity;
use crate::actions::Action;
use crate::game::Game;
use crate::errors::SimulationError;
use rand::Rng;
use std::any::Any;

pub struct Animal {
    pub id: u32,
    pub health: i32,
    pub species: String,
}

impl Entity for Animal {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_position(&self) -> (u32, u32) {
        // This will be handled by the Position component
        (0, 0)
    }

    fn get_health(&self) -> i32 {
        self.health
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }

    fn update(&mut self, _game: &Game) -> Result<Option<Action>, SimulationError> {
        // Simple random movement for now
        let mut rng = rand::thread_rng();
        let direction = match rng.gen_range(0..4) {
            0 => "up",
            1 => "down",
            2 => "left",
            _ => "right",
        };

        // In the future, this would return a Move action.
        // For now, we will just print the direction.
        println!("Animal {} wants to move {}", self.id, direction);

        Ok(None)
    }
}
