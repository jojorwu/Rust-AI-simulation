use bevy_ecs::prelude::*;
use crate::brain::BrainAction;
use crate::components::{intents::IntendsToCraft, BrainComponent, WantsToCraft};
use crate::errors::SimulationError;
use super::apply_brain_action;

pub fn craft_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &IntendsToCraft)>,
) {
    for (entity, mut brain_component, intent) in query.iter_mut() {
        let item_name = &intent.0;
        let result = execute_craft_item_goal(item_name);

        if let Ok(Some(action)) = result {
            apply_brain_action(&mut commands, entity, action);
        }

        // Crafting is a single-tick action, so the goal is complete.
        brain_component.current_goal = None;
        commands.entity(entity).remove::<IntendsToCraft>();
    }
}

fn execute_craft_item_goal(
    item_name: &str,
) -> Result<Option<BrainAction>, SimulationError> {
    Ok(Some(BrainAction::Craft(WantsToCraft {
        item_name: item_name.to_string(),
    })))
}
