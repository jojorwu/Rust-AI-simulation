use bevy_ecs::prelude::*;
use crate::brain::BrainAction;
use crate::components::{intents::IntendsToFlee, BrainComponent, Velocity};
use crate::errors::SimulationError;
use super::apply_brain_action;
use rand::Rng;

pub fn flee_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent), With<IntendsToFlee>>,
) {
    for (entity, mut brain_component) in query.iter_mut() {
        let result = execute_flee_goal();
        if let Ok(Some(action)) = result {
            apply_brain_action(&mut commands, entity, action);
        }

        // Fleeing is a single-tick action.
        brain_component.current_goal = None;
        commands.entity(entity).remove::<IntendsToFlee>();
    }
}

fn execute_flee_goal() -> Result<Option<BrainAction>, SimulationError> {
    let mut rng = rand::rng();
    let dx = rng.random_range(-1..=1);
    let dy = rng.random_range(-1..=1);
    Ok(Some(BrainAction::Move(Velocity { dx, dy })))
}
