use crate::brain::{Goal, HighLevelState};
use crate::components::Position;
use crate::map::Tile;
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, PartialEq, Event)]
pub enum Event {
    ChunkGenerated {
        position: (u32, u32),
        tiles: Vec<Vec<Tile>>,
    },
    EntityDied(Entity),
    FoundationBuilt {
        builder: Entity,
        position: Position,
    },
    GoalCompleted {
        entity: Entity,
        prev_state: HighLevelState,
        goal: Goal,
        new_state: HighLevelState,
        reward: f64,
    },
}
