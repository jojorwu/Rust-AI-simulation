use super::apply_brain_action;
use crate::brain::BrainAction;
use crate::components::{BrainComponent, WantsToAttack, intents::IntendsToAttack};
use crate::errors::SimulationError;
use bevy_ecs::prelude::*;

pub fn attack_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &IntendsToAttack)>,
) {
    for (entity, mut brain_component, intent) in query.iter_mut() {
        let target_id = intent.0;
        let result = execute_attack_goal(target_id);

        if let Ok(Some(action)) = result {
            apply_brain_action(&mut commands, entity, action);
        }

        // Attacking is a single-tick action for now.
        brain_component.current_goal = None;
        commands.entity(entity).remove::<IntendsToAttack>();
    }
}

fn execute_attack_goal(target_id: Entity) -> Result<Option<BrainAction>, SimulationError> {
    Ok(Some(BrainAction::Attack(WantsToAttack {
        target: target_id,
    })))
}
