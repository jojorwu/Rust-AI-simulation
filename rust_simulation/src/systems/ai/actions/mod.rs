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
use crate::components::{Velocity, WantsToAttack, WantsToBuild, WantsToCraft, WantsToGather, WantsToStoreItem};

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
