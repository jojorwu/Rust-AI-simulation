use bevy_ecs::prelude::*;

/// A component that identifies an entity as a player agent in the simulation.
#[derive(Component, Debug, Clone)]
pub struct Player {
    /// A unique identifier for the player.
    pub id: u32,
    /// The item currently held by the player, if any.
    pub held_item: Option<String>,
}

impl Player {
    /// Creates a new player with the given ID.
    pub fn new(id: u32, _map_width: u32, _map_height: u32) -> Self {
        Player { id, held_item: None }
    }

    /// Resets the player's state, such as clearing the held item.
    pub fn reset(&mut self) {
        self.held_item = None;
    }
}
