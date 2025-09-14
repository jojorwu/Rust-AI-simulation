//! This module defines components used for pathfinding.

use crate::async_task::PathfindingResult;
use bevy_ecs::prelude::*;
use bevy_tasks::Task;
use std::collections::VecDeque;

/// A component that signals a request for a path to be calculated.
///
/// This component is added to an entity to trigger the `pathfinding_system`.
#[derive(Component)]
pub struct PathRequest {
    /// The starting coordinate for the path.
    pub start: (u32, u32),
    /// The destination coordinate for the path.
    pub goal: (u32, u32),
}

/// A component that holds the calculated path for an entity to follow.
///
/// The path is a queue of nodes, where each node is a coordinate on the map.
#[derive(Component)]
pub struct CurrentPath {
    /// The sequence of coordinates that form the path.
    pub nodes: VecDeque<(u32, u32)>,
}

/// A component that holds the async `Task` for a pathfinding calculation.
///
/// This allows the pathfinding to run in the background without blocking the main thread.
#[derive(Component)]
pub struct PathfindingTask(pub Task<PathfindingResult>);

/// A marker component that signals that the last pathfinding attempt for this entity failed.
#[derive(Component)]
pub struct PathfindingFailed;
