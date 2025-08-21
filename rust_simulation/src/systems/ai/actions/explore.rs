use bevy_ecs::prelude::*;
use crate::brain::{BrainAction, Goal};
use crate::components::{BrainComponent, Position};
use crate::errors::SimulationError;
use crate::pathfinding;
use super::{apply_brain_action, follow_path};

pub fn explore_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &Position)>,
) {
    for (entity, mut brain_component, position) in query.iter_mut() {
        if let Some(Goal::Explore) = &brain_component.current_goal {
            if let Some(action) = follow_path(&mut brain_component, position) {
                apply_brain_action(&mut commands, entity, action);
                continue;
            }

            let result = execute_explore_goal(&mut brain_component, position);
             if let Ok(Some(action)) = result {
                apply_brain_action(&mut commands, entity, action);
            }
        }
    }
}

fn execute_explore_goal(
    brain_component: &mut BrainComponent,
    player_pos: &Position,
) -> Result<Option<BrainAction>, SimulationError> {
    if brain_component.current_path.is_some() {
        return Ok(None);
    }

    // Get the next destination from the frontier
    if let Some(target_pos) = brain_component.exploration_frontier.pop_front() {
        // If the target is already explored (e.g., seen while pathing), get a new one.
        if brain_component.mental_map[target_pos.y as usize][target_pos.x as usize].is_some() {
            // This could be recursive, but a simple loop is safer to avoid stack overflow.
            // For now, we'll just try again on the next tick by returning Ok(None).
            brain_component.exploration_frontier.push_back(target_pos); // Add it back to try later
            return Ok(None);
        }

        if let Some(path) = pathfinding::find_path(
            (player_pos.x, player_pos.y),
            (target_pos.x, target_pos.y),
            &brain_component.mental_map,
        ) {
            brain_component.current_path = Some(path);
        } else {
            // Could not find a path to the frontier, maybe it's unreachable.
            // Add it back to the end of the queue to try later.
            brain_component.exploration_frontier.push_back(target_pos);
        }
    } else {
        // No more frontiers to explore. The goal is complete.
        brain_component.current_goal = None;
    }

    Ok(None)
}
