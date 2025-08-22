pub mod attack;
pub mod craft;
pub mod explore;
pub mod flee;
pub mod stockpile;

// This module will contain the individual action systems
// and shared helper functions.

use bevy_ecs::prelude::*;
use crate::brain::BrainAction;
use crate::components::{Velocity, WantsToAttack, WantsToCraft, WantsToStoreItem};

pub fn apply_brain_action(commands: &mut Commands, entity: Entity, action: BrainAction) {
    match action {
        BrainAction::Move(vel) => {
            commands.entity(entity).insert(vel);
        }
        BrainAction::Craft(wants) => {
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
