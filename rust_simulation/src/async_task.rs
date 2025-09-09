//! This module defines the data structures used to pass results back
//! from asynchronous tasks to the main Bevy application.

use bevy_ecs::prelude::*;
use std::collections::VecDeque;

/// The result of an asynchronous pathfinding calculation.
///
/// This struct is sent from the background task back to the main thread
/// once the calculation is complete.
#[derive(Debug)]
pub struct PathfindingResult {
    /// The entity for which the path was calculated.
    pub entity: Entity,
    /// The calculated path, if one was found.
    /// `None` indicates that no path exists between the start and goal.
    pub path: Option<VecDeque<(u32, u32)>>,
}
