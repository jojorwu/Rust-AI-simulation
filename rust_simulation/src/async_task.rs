use bevy_ecs::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum AsyncResult {
    Pathfinding(PathfindingResult),
}

#[derive(Debug)]
pub struct PathfindingResult {
    pub entity: Entity,
    pub path: Option<VecDeque<(u32, u32)>>,
}

#[derive(Resource)]
pub struct AsyncResultChannel {
    pub sender: Sender<AsyncResult>,
    pub receiver: Receiver<AsyncResult>,
}

impl Default for AsyncResultChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}
