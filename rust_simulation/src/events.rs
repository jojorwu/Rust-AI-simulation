use crate::components::Position;
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, PartialEq, Event)]
pub enum Event {
    EntityDied(Entity),
    FoundationBuilt { builder: Entity, position: Position },
}
