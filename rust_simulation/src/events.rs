use crate::brain::{Goal, HighLevelState};
use crate::components::Position;
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, PartialEq, Event)]
pub enum Event {
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
