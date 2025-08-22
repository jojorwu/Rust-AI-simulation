//! Components related to pathfinding.

use bevy_ecs::prelude::*;
use std::collections::VecDeque;

/// A component that signals a request for a path to be calculated.
#[derive(Component)]
pub struct PathRequest {
    pub start: (u32, u32),
    pub goal: (u32, u32),
}

/// A component that holds the calculated path for an entity to follow.
#[derive(Component)]
pub struct CurrentPath {
    pub nodes: VecDeque<(u32, u32)>,
}
