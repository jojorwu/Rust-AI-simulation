use crate::brain::{BrainAction, Goal, RelationshipStatus};
use crate::components::{
    BrainComponent, Position, Velocity, WantsToAttack, WantsToBuild, WantsToCraft, WantsToGather,
    WantsToStoreItem, Chest,
};
use crate::map::Map;
use crate::BrainResource;
use bevy_ecs::prelude::*;
use crate::errors::SimulationError;
use crate::pathfinding;
use rand::Rng;

pub fn action_execution_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &Position)>,
    brain_res: Res<BrainResource>,
    map: Res<Map>,
    chest_query: Query<(Entity, &Position, &Chest)>,
) {
    let brain = &brain_res.0;
    for (entity, mut brain_component, position) in query.iter_mut() {
        if let Some(action) = follow_path(&mut brain_component, position) {
            apply_brain_action(&mut commands, entity, action);
            continue;
        }

        if let Some(goal) = brain_component.current_goal.clone() {
            let result = match goal {
                Goal::GatherResource(name) => execute_gather_goal(
                    &mut brain_component,
                    &map,
                    entity,
                    position,
                    &name,
                ),
                Goal::CraftItem(name) => execute_craft_item_goal(&name),
                Goal::Build(name) => execute_build_goal(&name),
                Goal::Attack(id) => execute_attack_goal(id),
                Goal::Flee => execute_flee_goal(&mut brain_component, position),
                Goal::Explore => execute_explore_goal(&mut brain_component, position),
                Goal::Stockpile(res) => execute_stockpile_goal(
                    &mut brain_component,
                    &chest_query,
                    position,
                    &res,
                ),
            };

            if let Ok(Some(action)) = result {
                apply_brain_action(&mut commands, entity, action);
            }
        }
    }
}

fn apply_brain_action(commands: &mut Commands, entity: Entity, action: BrainAction) {
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

fn follow_path(
    brain_component: &mut BrainComponent,
    position: &Position,
) -> Option<BrainAction> {
    if let Some(path) = &mut brain_component.current_path {
        if !path.is_empty() {
            let next_pos = path.remove(0);
            let (dx, dy) = (
                next_pos.0 as i32 - position.x as i32,
                next_pos.1 as i32 - position.y as i32,
            );
            return Some(BrainAction::Move(Velocity { dx, dy }));
        } else {
            brain_component.current_path = None;
        }
    }
    None
}

fn execute_gather_goal(
    brain_component: &mut BrainComponent,
    map: &Map,
    entity: Entity,
    player_pos: &Position,
    resource_name: &str,
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
                // Simplified: just assume the first entity at the position is the target
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
                    return Ok(None);
                }
            }
        }
    }
    brain_component.current_goal = None;
    Ok(None)
}

fn execute_craft_item_goal(
    item_name: &str,
) -> Result<Option<BrainAction>, SimulationError> {
    Ok(Some(BrainAction::Craft(WantsToCraft {
        item_name: item_name.to_string(),
    })))
}

fn execute_build_goal(
    structure_name: &str,
) -> Result<Option<BrainAction>, SimulationError> {
    Ok(Some(BrainAction::Build(WantsToBuild {
        structure_name: structure_name.to_string(),
    })))
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

fn execute_flee_goal(
    brain_component: &mut BrainComponent,
    _player_pos: &Position,
) -> Result<Option<BrainAction>, SimulationError> {
    let mut rng = rand::thread_rng();
    let dx = rng.gen_range(-1..=1);
    let dy = rng.gen_range(-1..=1);
    brain_component.current_goal = None; // Flee for one tick
    Ok(Some(BrainAction::Move(Velocity { dx, dy })))
}

fn execute_explore_goal(
    brain_component: &mut BrainComponent,
    player_pos: &Position,
) -> Result<Option<BrainAction>, SimulationError> {
    if brain_component.current_path.is_some() {
        return Ok(None);
    }
    let mut unvisited = Vec::new();
    for y in 0..crate::config::HEIGHT {
        for x in 0..crate::config::WIDTH {
            if brain_component.mental_map[y as usize][x as usize].is_none() {
                unvisited.push((x, y));
            }
        }
    }
    if !unvisited.is_empty() {
        let mut rng = rand::thread_rng();
        let target_idx = rng.gen_range(0..unvisited.len());
        let target_pos = unvisited[target_idx];
        if let Some(path) = pathfinding::find_path(
            (player_pos.x, player_pos.y),
            target_pos,
            &brain_component.mental_map,
        ) {
            brain_component.current_path = Some(path);
        }
    }
    Ok(None)
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
