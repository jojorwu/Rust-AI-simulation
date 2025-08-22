use crate::brain::{Goal, HighLevelState, InventorySummary};
use crate::components::{
    ai::{GoalQTable, KnownResources, PlayerMemories},
    intents::*,
    BrainComponent, Equipped, Health, Inventory,
};
use crate::config;
use crate::errors::SimulationError;
use crate::IsDay;
use bevy_ecs::prelude::*;
use log::info;
use rand::Rng;
use std::collections::HashMap;

pub fn goal_selection_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut BrainComponent,
        &Health,
        &Inventory,
        &KnownResources,
        &PlayerMemories,
        &GoalQTable,
        &Equipped,
    )>,
    is_day: Res<IsDay>,
) {
    let mut rng = rand::thread_rng();

    for (
        entity,
        mut brain,
        health,
        inventory,
        known_resources,
        player_memories,
        goal_q_table,
        equipped,
    ) in query.iter_mut()
    {
        if brain.current_goal.is_none() && brain.goal_commitment_ticks == 0 {
            let high_level_state =
                get_high_level_state(health, inventory, player_memories, equipped, is_day.0);

            if let Ok(new_high_level_goal) = choose_goal(
                &high_level_state,
                &brain,
                known_resources,
                goal_q_table,
                is_day.0,
                &mut rng,
            ) {
                if let Ok(mut plan) =
                    plan_goal(&brain, inventory, known_resources, &new_high_level_goal, equipped)
                {
                    plan.reverse();
                    brain.goal_stack = plan;
                    brain.current_goal = brain.goal_stack.pop();
                    if let Some(goal) = &brain.current_goal {
                        info!("Entity {:?} selected new goal: {:?}", entity, goal);

                        match goal {
                            Goal::GatherResource(res) => {
                                commands.entity(entity).insert(IntendsToGather(res.clone()));
                            }
                            Goal::CraftItem(item) => {
                                commands.entity(entity).insert(IntendsToCraft(item.clone()));
                            }
                            Goal::Build(structure) => {
                                commands.entity(entity).insert(IntendsToBuild(structure.clone()));
                            }
                            Goal::Attack(target) => {
                                commands.entity(entity).insert(IntendsToAttack(*target));
                            }
                            Goal::Equip(tool) => {
                                commands.entity(entity).insert(IntendsToEquip(tool.clone()));
                            }
                            Goal::Flee => {
                                commands.entity(entity).insert(IntendsToFlee);
                            }
                            Goal::Explore => {
                                commands.entity(entity).insert(IntendsToExplore);
                            }
                            Goal::Stockpile(res) => {
                                commands.entity(entity).insert(IntendsToStockpile(res.clone()));
                            }
                        }

                        brain.state_at_goal_start = Some(high_level_state);
                        brain.goal_commitment_ticks = config::GOAL_COMMITMENT_TICKS;
                    }
                }
            }
        }
    }
}

use crate::brain::ResourceLevel;

pub fn get_high_level_state(
    health: &Health,
    inventory: &Inventory,
    player_memories: &PlayerMemories,
    equipped: &Equipped,
    is_day: bool,
) -> HighLevelState {
    let num_hostile_players = player_memories
        .0
        .values()
        .filter(|m| m.relationship == crate::brain::RelationshipStatus::Hostile)
        .count() as u32;

    let get_resource_level = |quantity: u32| {
        if quantity == 0 {
            ResourceLevel::None
        } else if quantity < config::RESOURCE_LEVEL_LOW_THRESHOLD {
            ResourceLevel::Low
        } else {
            ResourceLevel::High
        }
    };

    let inventory_summary = InventorySummary {
        wood_level: get_resource_level(inventory.get_quantity("wood")),
        stone_level: get_resource_level(inventory.get_quantity("stone")),
        iron_ore_level: get_resource_level(inventory.get_quantity("iron_ore")),
        has_stone_axe: inventory.has_item("stone_axe", 1),
    };

    HighLevelState {
        inventory_summary,
        equipped_tool: equipped.tool.clone(),
        num_hostile_players,
        health_level: health.current as u32,
        is_night: !is_day,
    }
}

fn choose_goal(
    state: &HighLevelState,
    brain: &BrainComponent,
    known_resources: &KnownResources,
    goal_q_table: &GoalQTable,
    is_night: bool,
    rng: &mut impl Rng,
) -> Result<Goal, SimulationError> {
    if state.health_level < config::FLEE_HEALTH_THRESHOLD {
        return Ok(Goal::Flee);
    }

    let valid_goals: Vec<_> = brain
        .goals
        .iter()
        .filter(|g| is_goal_valid(g, known_resources))
        .cloned()
        .collect();
    if valid_goals.is_empty() {
        return Ok(Goal::Flee);
    }

    if rng.r#gen::<f64>() < brain.epsilon {
        let index = rng.r#gen_range(0..valid_goals.len());
        return Ok(valid_goals[index].clone());
    }

    if let Some(q_values) = goal_q_table.0.get(state) {
        q_values
            .iter()
            .filter(|(g, _)| is_goal_valid(g, known_resources))
            .map(|(goal, q_value)| {
                let effective_q_value = if is_night {
                    if let Goal::Build(_) = goal {
                        *q_value + config::BUILD_GOAL_BONUS
                    } else {
                        *q_value
                    }
                } else {
                    *q_value
                };
                (goal, effective_q_value)
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(goal, _)| goal.clone())
            .map(Ok)
            .unwrap_or_else(|| {
                let index = rng.r#gen_range(0..valid_goals.len());
                Ok(valid_goals[index].clone())
            })
    } else {
        let index = rng.r#gen_range(0..valid_goals.len());
        Ok(valid_goals[index].clone())
    }
}

fn is_goal_valid(goal: &Goal, known_resources: &KnownResources) -> bool {
    match goal {
        Goal::GatherResource(resource_name) => known_resources
            .0
            .get(resource_name)
            .map_or(false, |s| !s.is_empty()),
        _ => true,
    }
}

fn plan_goal(
    brain: &BrainComponent,
    inventory: &Inventory,
    known_resources: &KnownResources,
    goal: &Goal,
    equipped: &Equipped,
) -> Result<Vec<Goal>, SimulationError> {
    let mut plan = Vec::new();
    match goal {
        Goal::GatherResource(resource) => {
            // Find the best tool in inventory for this resource.
            let best_tool = inventory
                .items
                .keys()
                .filter_map(|item_name| {
                    brain
                        .item_registry
                        .get_item(item_name)
                        .and_then(|item_def| {
                            if let Some(improves) = &item_def.improves_gathering {
                                if improves.contains(resource) {
                                    return Some((item_name, item_def.tier));
                                }
                            }
                            None
                        })
                })
                .max_by_key(|&(_, tier)| tier)
                .map(|(name, _)| name);

            // If we found a best tool and it's not equipped, plan to equip it.
            if let Some(tool) = best_tool {
                if equipped.tool.as_deref() != Some(tool) {
                    plan.push(Goal::Equip(tool.clone()));
                }
            }
            plan.push(goal.clone());
        }
        Goal::CraftItem(item_name) => {
            let required = brain
                .recipe_manager
                .get_required_resources(item_name, 1);
            plan.extend(plan_resource_gathering(
                inventory,
                known_resources,
                &required,
            ));
            plan.push(goal.clone());
        }
        Goal::Build(structure_name) => {
            let required = brain
                .recipe_manager
                .get_required_resources(structure_name, 1);
            plan.extend(plan_resource_gathering(
                inventory,
                known_resources,
                &required,
            ));
            plan.push(goal.clone());
        }
        Goal::Stockpile(resource) => {
            let has_enough = inventory.has_item(resource, 1);
            if !has_enough {
                plan.push(Goal::GatherResource(resource.clone()));
            }
            plan.push(goal.clone());
        }
        _ => {
            plan.push(goal.clone());
        }
    }
    Ok(plan)
}

fn plan_resource_gathering(
    inventory: &Inventory,
    known_resources: &KnownResources,
    required: &HashMap<String, u32>,
) -> Vec<Goal> {
    let mut plan = Vec::new();
    for (resource, &required_amount) in required {
        let has_enough = inventory.get_quantity(resource) >= required_amount;
        if !has_enough {
            if !known_resources.0.contains_key(resource) {
                plan.push(Goal::Explore);
            }
            plan.push(Goal::GatherResource(resource.clone()));
        }
    }
    plan
}
