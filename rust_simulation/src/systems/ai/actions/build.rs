use bevy_ecs::prelude::*;
use crate::brain::BrainAction;
use crate::components::{intents::IntendsToBuild, BrainComponent, WantsToBuild};
use crate::errors::SimulationError;
use super::apply_brain_action;

pub fn build_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &IntendsToBuild)>,
) {
    for (entity, mut brain_component, intent) in query.iter_mut() {
        let structure_name = &intent.0;
        let result = execute_build_goal(structure_name);

        if let Ok(Some(action)) = result {
            apply_brain_action(&mut commands, entity, action);
        }

        // Building is a single-tick action for now.
        brain_component.current_goal = None;
        commands.entity(entity).remove::<IntendsToBuild>();
    }
}

fn execute_build_goal(
    structure_name: &str,
) -> Result<Option<BrainAction>, SimulationError> {
    Ok(Some(BrainAction::Build(WantsToBuild {
        structure_name: structure_name.to_string(),
    })))
}
