use crate::brain::{Goal, HighLevelState, InventorySummary};
use crate::components::{
    ai::{GoalQTable, KnownResources, PlayerMemories},
    intents::*,
    BrainComponent, Health, Inventory,
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
    ) in query.iter_mut()
    {
        if brain.current_goal.is_none() && brain.goal_commitment_ticks == 0 {
            let high_level_state =
                get_high_level_state(health, inventory, player_memories, is_day.0);

            if let Ok(new_high_level_goal) = choose_goal(
                &high_level_state,
                &brain,
                known_resources,
                goal_q_table,
                is_day.0,
                &mut rng,
            ) {
                if let Ok(mut plan) =
                    plan_goal(&brain, inventory, known_resources, &new_high_level_goal)
                {
                    plan.reverse();
                    brain.goal_stack = plan;
                    brain.current_goal = brain.goal_stack.pop();
                    if let Some(goal) = &brain.current_goal {
                        info!("Entity {:?} selected new goal: {:?}", entity, goal);

                        // Add the corresponding intent component
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

                        brain.goal_commitment_ticks = config::GOAL_COMMITMENT_TICKS;
                    }
                }
            }
        }
    }
}

/// Constructs the high-level state of an agent from its components.
fn get_high_level_state(
    health: &Health,
    inventory: &Inventory,
    player_memories: &PlayerMemories,
    is_day: bool,
) -> HighLevelState {
    let num_hostile_players = player_memories
        .0
        .values()
        .filter(|m| m.relationship == crate::brain::RelationshipStatus::Hostile)
        .count() as u32;

    let inventory_summary = InventorySummary {
        has_wood: inventory.has_item("wood", 1),
        has_stone: inventory.has_item("stone", 1),
        has_iron_ore: inventory.has_item("iron_ore", 1),
        has_stone_axe: inventory.has_item("stone_axe", 1),
    };

    HighLevelState {
        inventory_summary,
        num_hostile_players,
        health_level: health.current as u32,
        is_night: !is_day,
    }
}

/// Chooses a high-level goal for the agent based on the current state.
fn choose_goal(
    state: &HighLevelState,
    brain: &BrainComponent,
    known_resources: &KnownResources,
    goal_q_table: &GoalQTable,
    is_night: bool,
    rng: &mut impl Rng,
) -> Result<Goal, SimulationError> {
    const FLEE_HEALTH_THRESHOLD: u32 = 25;
    if state.health_level < FLEE_HEALTH_THRESHOLD {
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

/// Checks if a goal is currently valid.
fn is_goal_valid(goal: &Goal, known_resources: &KnownResources) -> bool {
    match goal {
        Goal::GatherResource(resource_name) => known_resources
            .0
            .get(resource_name)
            .map_or(false, |s| !s.is_empty()),
        _ => true,
    }
}

/// Creates a plan (a sequence of sub-goals) to achieve a given high-level goal.
fn plan_goal(
    brain: &BrainComponent,
    inventory: &Inventory,
    known_resources: &KnownResources,
    goal: &Goal,
) -> Result<Vec<Goal>, SimulationError> {
    let mut plan = Vec::new();
    match goal {
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

/// Plans the gathering of resources required for a crafting recipe.
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
