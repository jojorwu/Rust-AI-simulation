use crate::brain::{Goal, HighLevelState, MemoryTile, PlayerMemory};
use crate::components::Position;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// A component representing the agent's memory of the map layout.
#[derive(Component, Clone)]
pub struct MentalMap(pub Vec<Vec<Option<MemoryTile>>>);

/// A component representing the agent's knowledge of resource locations.
#[derive(Component, Clone)]
pub struct KnownResources(pub HashMap<String, HashSet<Position>>);

/// A component representing the agent's memories of other players.
#[derive(Component, Clone)]
pub struct PlayerMemories(pub HashMap<Entity, PlayerMemory>);

/// A component representing the agent's Q-table for goal selection.
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct GoalQTable(pub Vec<(HighLevelState, HashMap<Goal, f64>)>);

/// A component representing the agent's frontier for exploration.
#[derive(Component, Clone)]
pub struct ExplorationFrontier(pub VecDeque<Position>);
