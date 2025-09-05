use bevy_ecs::prelude::*;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct PathfindingResult {
    pub entity: Entity,
    pub path: Option<VecDeque<(u32, u32)>>,
}
