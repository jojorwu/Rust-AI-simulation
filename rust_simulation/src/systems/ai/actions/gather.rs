use bevy_ecs::prelude::*;
use crate::brain::BrainAction;
use crate::components::{intents::IntendsToGather, BrainComponent, Position, WantsToGather};
use crate::errors::SimulationError;
use crate::map::Map;
use crate::pathfinding;
use super::{apply_brain_action, follow_path};

pub fn gather_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &Position, &IntendsToGather)>,
    map: Res<Map>,
) {
    for (entity, mut brain_component, position, intent) in query.iter_mut() {
        let resource_name = &intent.0;

        if let Some(action) = follow_path(&mut brain_component, position) {
            apply_brain_action(&mut commands, entity, action);
            continue;
        }

        let result = execute_gather_goal(&mut brain_component, &map, resource_name, position);
        if let Ok(Some(action)) = result {
            apply_brain_action(&mut commands, entity, action);
            // The goal is not complete until the gathering is done,
            // but the intent to *start* gathering is complete.
            // The `gathering_system` will eventually complete the `current_goal`.
            commands.entity(entity).remove::<IntendsToGather>();
        } else {
            // If execute_gather_goal returns Ok(None) or an Error, it means we can't
            // currently pursue this. Remove the intent to allow for replanning.
            commands.entity(entity).remove::<IntendsToGather>();
            brain_component.current_goal = None;
        }
    }
}

fn execute_gather_goal(
    brain_component: &mut BrainComponent,
    map: &Map,
    resource_name: &str,
    player_pos: &Position,
) -> Result<Option<BrainAction>, SimulationError> {
    if let Some(known_positions) = brain_component.known_resources.get(resource_name) {
        let mut sorted_positions: Vec<_> = known_positions.iter().collect();
        sorted_positions
            .sort_by_key(|pos| pos.x.abs_diff(player_pos.x) + pos.y.abs_diff(player_pos.y));
        if let Some(target_pos) = sorted_positions.first() {
            let (dx, dy) = (
                (player_pos.x as i32 - target_pos.x as i32).abs(),
                (player_pos.y as i32 - target_pos.y as i32).abs(),
            );
            if dx <= 1 && dy <= 1 {
                if let Some(target_entity) = map.get_entities_at(target_pos.x, target_pos.y).and_then(|v| v.first().copied()) {
                    return Ok(Some(BrainAction::Gather(WantsToGather {
                        target: target_entity,
                    })));
                }
            } else if brain_component.current_path.is_none() {
                if let Some(path) = pathfinding::find_path(
                    (player_pos.x, player_pos.y),
                    (target_pos.x, target_pos.y),
                    &brain_component.mental_map,
                ) {
                    brain_component.current_path = Some(path);
                    return Ok(None); // Pathing, no immediate action
                }
            }
        }
    }
    // If we reach here, we can't find the resource, so the goal is impossible.
    Ok(None)
}
