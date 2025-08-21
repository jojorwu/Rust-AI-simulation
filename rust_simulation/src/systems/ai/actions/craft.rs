use bevy_ecs::prelude::*;
use crate::brain::{BrainAction, Goal};
use crate::components::{BrainComponent, WantsToCraft};
use crate::errors::SimulationError;
use super::apply_brain_action;

pub fn craft_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent)>,
) {
    for (entity, mut brain_component) in query.iter_mut() {
        if let Some(Goal::CraftItem(item_name)) = &brain_component.current_goal {
            let result = execute_craft_item_goal(item_name);
            if let Ok(Some(action)) = result {
                apply_brain_action(&mut commands, entity, action);
            }
            // Crafting is a single-tick action, so the goal is complete.
            brain_component.current_goal = None;
        }
    }
}

fn execute_craft_item_goal(
    item_name: &str,
) -> Result<Option<BrainAction>, SimulationError> {
    Ok(Some(BrainAction::Craft(WantsToCraft {
        item_name: item_name.to_string(),
    })))
}
