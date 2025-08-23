use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Player {
    pub id: u32,
    pub _held_item: Option<String>,
}

impl Player {
    pub fn new(id: u32, _map_width: u32, _map_height: u32) -> Self {
        Player { id, _held_item: None }
    }

    pub fn reset(&mut self) {
        self._held_item = None;
    }
}
