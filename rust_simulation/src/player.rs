use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Player {
    pub _held_item: Option<String>,
}

impl Player {
    pub fn new(_id: u32, _map_width: u32, _map_height: u32) -> Self {
        Player {
            _held_item: None,
        }
    }

    pub fn reset(&mut self) {
        self._held_item = None;
    }
}
