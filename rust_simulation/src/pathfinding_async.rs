use bevy_ecs::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use std::collections::VecDeque;

/// The result of a pathfinding calculation.
/// This is sent from the background thread to the main thread.
#[derive(Debug)]
pub struct PathfindingResult {
    pub entity: Entity,
    pub path: Option<VecDeque<(u32, u32)>>,
}

/// A resource that holds the MPSC channel for pathfinding results.
#[derive(Resource)]
pub struct PathfindingResultChannel {
    pub sender: Sender<PathfindingResult>,
    pub receiver: Receiver<PathfindingResult>,
}

impl Default for PathfindingResultChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}
