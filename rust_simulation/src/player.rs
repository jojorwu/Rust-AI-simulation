use super::map::MentalMap;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Player {
    pub _held_item: Option<String>,
    pub mental_map: MentalMap,
}

impl Player {
    pub fn new(_id: u32, map_width: u32, map_height: u32) -> Self {
        Player {
            _held_item: None,
            mental_map: MentalMap::new(map_width, map_height),
        }
    }

    pub fn reset(&mut self) {
        self._held_item = None;
    }
}
