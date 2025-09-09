use crate::brain::{Goal, HighLevelState, MemoryTile, PlayerMemory};
use crate::components::Position;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// A component representing the agent's memory of the map layout.
///
/// The `MentalMap` is a 2D grid that stores the agent's knowledge of the world tiles.
/// Tiles that have not been observed are `None`.
#[derive(Component, Clone)]
pub struct MentalMap(pub Vec<Vec<Option<MemoryTile>>>);

/// A component representing the agent's knowledge of resource locations.
///
/// This map stores a set of known positions for each type of resource.
#[derive(Component, Clone)]
pub struct KnownResources(pub HashMap<String, HashSet<Position>>);

/// A component representing the agent's memories of other players.
///
/// This map stores information about other entities, such as their relationship status.
#[derive(Component, Clone)]
pub struct PlayerMemories(pub HashMap<Entity, PlayerMemory>);

/// A component representing the agent's Q-table for goal selection.
///
/// The Q-table maps a `HighLevelState` to a set of `Goal`s and their associated Q-values,
/// which represent the expected future reward for choosing that goal in that state.
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct GoalQTable(
    /// The nested HashMap representing the Q-table.
    #[serde(with = "crate::serde_helpers::q_table_map_format")]
    pub HashMap<HighLevelState, HashMap<Goal, f64>>,
);

/// A component representing the agent's frontier for exploration.
///
/// This is a queue of positions that the agent has discovered but not yet visited,
/// driving the exploration behavior.
#[derive(Component, Clone)]
pub struct ExplorationFrontier(pub VecDeque<Position>);
