pub mod attack;
pub mod build;
pub mod craft;
pub mod explore;
pub mod flee;
pub mod gather;
pub mod stockpile;

// This module will contain the individual action systems
// and shared helper functions.

use bevy_ecs::prelude::*;
use crate::brain::BrainAction;
use crate::components::{BrainComponent, Position, Velocity};

pub fn apply_brain_action(commands: &mut Commands, entity: Entity, action: BrainAction) {
    match action {
        BrainAction::Move(vel) => {
            commands.entity(entity).insert(vel);
        }
        BrainAction::Gather(wants) => {
            commands.entity(entity).insert(wants);
        }
        BrainAction::Craft(wants) => {
            commands.entity(entity).insert(wants);
        }
        BrainAction::Build(wants) => {
            commands.entity(entity).insert(wants);
        }
        BrainAction::Attack(wants) => {
            commands.entity(entity).insert(wants);
        }
        BrainAction::Store(wants) => {
            commands.entity(entity).insert(wants);
        }
    }
}

pub fn follow_path(
    brain_component: &mut BrainComponent,
    position: &Position,
) -> Option<BrainAction> {
    if let Some(path) = &mut brain_component.current_path {
        if !path.is_empty() {
            let next_pos = path.remove(0);
            let (dx, dy) = (
                next_pos.0 as i32 - position.x as i32,
                next_pos.1 as i32 - position.y as i32,
            );
            return Some(BrainAction::Move(Velocity { dx, dy }));
        } else {
            brain_component.current_path = None;
        }
    }
    None
}
