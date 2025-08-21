use bevy_ecs::prelude::*;
use crate::brain::{BrainAction, Goal};
use crate::components::{BrainComponent, WantsToAttack};
use crate::errors::SimulationError;
use super::apply_brain_action;

pub fn attack_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent)>,
) {
    for (entity, mut brain_component) in query.iter_mut() {
        if let Some(Goal::Attack(target_id)) = brain_component.current_goal {
            let result = execute_attack_goal(target_id);
            if let Ok(Some(action)) = result {
                apply_brain_action(&mut commands, entity, action);
            }
            // Attacking is a single-tick action for now.
            brain_component.current_goal = None;
        }
    }
}

fn execute_attack_goal(
    target_id: Entity,
) -> Result<Option<BrainAction>, SimulationError> {
    Ok(Some(BrainAction::Attack(
        WantsToAttack {
            target: target_id,
        },
    )))
}
