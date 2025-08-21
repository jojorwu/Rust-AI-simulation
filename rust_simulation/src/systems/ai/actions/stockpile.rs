use bevy_ecs::prelude::*;
use crate::brain::{BrainAction, Goal};
use crate::components::{BrainComponent, Position, WantsToStoreItem, Chest};
use crate::errors::SimulationError;
use crate::pathfinding;
use super::{apply_brain_action, follow_path};


pub fn stockpile_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &Position)>,
    chest_query: Query<(Entity, &Position, &Chest)>,
) {
    for (entity, mut brain_component, position) in query.iter_mut() {
        if let Some(Goal::Stockpile(resource_name)) = brain_component.current_goal.clone() {
            if let Some(action) = follow_path(&mut brain_component, position) {
                apply_brain_action(&mut commands, entity, action);
                continue;
            }

            let result = execute_stockpile_goal(&mut brain_component, &chest_query, position, &resource_name);
            if let Ok(Some(action)) = result {
                apply_brain_action(&mut commands, entity, action);
            }
        }
    }
}


fn execute_stockpile_goal(
    brain_component: &mut BrainComponent,
    chest_query: &Query<(Entity, &Position, &Chest)>,
    player_pos: &Position,
    resource: &str,
) -> Result<Option<BrainAction>, SimulationError> {
    let Some(home_base_pos) = brain_component.home_base else {
        brain_component.current_goal = None;
        return Ok(None);
    };
    if let Some((chest_entity, chest_pos)) = find_closest_chest(chest_query, &home_base_pos) {
        let (dx, dy) = (
            (player_pos.x as i32 - chest_pos.x as i32).abs(),
            (player_pos.y as i32 - chest_pos.y as i32).abs(),
        );
        if dx <= 1 && dy <= 1 {
            return Ok(Some(BrainAction::Store(WantsToStoreItem {
                item_name: resource.to_string(),
                quantity: 1, // Simplified
                target_chest: chest_entity,
            })));
        } else if brain_component.current_path.is_none() {
            if let Some(path) = pathfinding::find_path(
                (player_pos.x, player_pos.y),
                (chest_pos.x, chest_pos.y),
                &brain_component.mental_map,
            ) {
                brain_component.current_path = Some(path);
            }
        }
    } else {
        brain_component.current_goal = None;
    }
    Ok(None)
}

fn find_closest_chest(
    chest_query: &Query<(Entity, &Position, &Chest)>,
    pos: &Position,
) -> Option<(Entity, Position)> {
    chest_query
        .iter()
        .map(|(e, p, _c)| (e, *p))
        .min_by_key(|(_, chest_pos)| chest_pos.x.abs_diff(pos.x) + chest_pos.y.abs_diff(pos.y))
}
