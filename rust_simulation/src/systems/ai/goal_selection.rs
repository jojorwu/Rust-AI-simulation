use crate::brain::{Goal, HighLevelState, InventorySummary};
use crate::components::{
    ai::{GoalQTable, KnownResources, PlayerMemories},
    intents::*,
    status::{Health, Hunger},
    BrainComponent, Inventory,
};
use crate::config::Config;
use crate::errors::SimulationError;
use crate::map::Map;
use crate::IsDay;
use bevy::ecs::system::ParallelCommands;
use bevy_ecs::prelude::*;
use log::info;
use rand::Rng;
use std::collections::HashMap;

pub fn goal_selection_system(
    mut query: Query<(
        Entity,
        &mut BrainComponent,
        &Health,
        &Hunger,
        &Inventory,
        &KnownResources,
        &PlayerMemories,
        &GoalQTable,
    )>,
    is_day: Res<IsDay>,
    config: Res<Config>,
    map: Res<Map>,
) {
    query.par_iter_mut().for_each(
        |(entity, mut brain, health, hunger, inventory, known_resources, player_memories, goal_q_table)| {
            if brain.current_goal.is_none() && brain.goal_commitment_ticks == 0 {
                let high_level_state =
                    get_high_level_state(health, hunger, inventory, player_memories, is_day.0);

                let mut rng = rand::rng();
                if let Ok(new_high_level_goal) = choose_goal(
                    &high_level_state,
                    &brain,
                    inventory,
                    known_resources,
                    goal_q_table,
                    is_day.0,
                    &config,
                    &map,
                    &mut rng,
                ) {
                    brain.current_goal = Some(new_high_level_goal);
                    if let Some(goal) = &brain.current_goal {
                        info!("Entity {:?} selected new goal: {:?}", entity, goal);
                        brain.goal_commitment_ticks = config.ai.goals.commitment_ticks;
                    }
                }
            }
        },
    );
}

pub fn goal_planning_system(
    mut query: Query<(Entity, &mut BrainComponent, &Inventory, &KnownResources)>,
) {
    for (_entity, mut brain, inventory, known_resources) in query.iter_mut() {
        if let Some(goal) = &brain.current_goal {
            if brain.goal_stack.is_empty() {
                if let Ok(mut plan) = plan_goal(&brain, inventory, known_resources, goal) {
                    plan.reverse();
                    brain.goal_stack = plan;
                }
            }
        }
    }
}

pub fn intent_creation_system(
    commands: ParallelCommands,
    query: Query<(Entity, &BrainComponent)>,
) {
    query.par_iter().for_each(|(entity, brain)| {
        if let Some(goal) = &brain.current_goal {
            commands.command_scope(|mut c| match goal {
                Goal::GatherResource(res) => {
                    c.entity(entity).insert(IntendsToGather(res.clone()));
                }
                Goal::CraftItem(item) => {
                    c.entity(entity).insert(IntendsToCraft(item.clone()));
                }
                Goal::Build(structure) => {
                    c.entity(entity)
                        .insert(IntendsToBuild(structure.clone()));
                }
                Goal::Attack(target) => {
                    c.entity(entity).insert(IntendsToAttack(*target));
                }
                Goal::Flee => {
                    c.entity(entity).insert(IntendsToFlee);
                }
                Goal::Explore => {
                    c.entity(entity).insert(IntendsToExplore);
                }
                Goal::Stockpile(res) => {
                    c.entity(entity)
                        .insert(IntendsToStockpile(res.clone()));
                }
                Goal::EatFood(food) => {
                    c.entity(entity).insert(WantsToEat(food.clone()));
                }
            });
        }
    });
}

/// Constructs the high-level state of an agent from its components.
fn get_high_level_state(
    health: &Health,
    hunger: &Hunger,
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
        wood: inventory.get_quantity("wood"),
        stone: inventory.get_quantity("stone"),
        iron_ore: inventory.get_quantity("iron_ore"),
        stone_axe: inventory.get_quantity("stone_axe"),
    };

    HighLevelState {
        inventory_summary,
        num_hostile_players,
        health_level: health.current as u32,
        hunger_level: hunger.current as u32,
        is_night: !is_day,
    }
}

/// Chooses a high-level goal for the agent based on the current state.
fn choose_goal(
    state: &HighLevelState,
    brain: &BrainComponent,
    inventory: &Inventory,
    known_resources: &KnownResources,
    goal_q_table: &GoalQTable,
    is_night: bool,
    config: &Config,
    map: &Map,
    rng: &mut impl Rng,
) -> Result<Goal, SimulationError> {
    const FLEE_HEALTH_THRESHOLD: u32 = 25;
    if state.health_level < FLEE_HEALTH_THRESHOLD {
        return Ok(Goal::Flee);
    }

    const HUNGER_THRESHOLD: u32 = 50;
    if state.hunger_level < HUNGER_THRESHOLD {
        if inventory.has_item("meat", 1) {
            return Ok(Goal::EatFood("meat".to_string()));
        } else {
            return Ok(Goal::GatherResource("pig".to_string()));
        }
    }

    let valid_goals: Vec<_> = brain
        .goals
        .iter()
        .filter(|g| is_goal_valid(g, known_resources, map))
        .cloned()
        .collect();
    if valid_goals.is_empty() {
        return Ok(Goal::Flee);
    }

    if rng.random::<f64>() < brain.epsilon {
        let index = rng.random_range(0..valid_goals.len());
        return Ok(valid_goals[index].clone());
    }

    if let Some((_, q_values)) = goal_q_table.0.iter().find(|(s, _)| s == state) {
        q_values
            .iter()
            .filter(|(g, _)| is_goal_valid(g, known_resources, map))
            .map(|(goal, q_value)| {
                let effective_q_value = if is_night {
                    if let Goal::Build(_) = goal {
                        *q_value + config.ai.goals.build_bonus
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
                let index = rng.random_range(0..valid_goals.len());
                Ok(valid_goals[index].clone())
            })
    } else {
        let index = rng.random_range(0..valid_goals.len());
        Ok(valid_goals[index].clone())
    }
}

/// Checks if a goal is currently valid.
fn is_goal_valid(goal: &Goal, known_resources: &KnownResources, map: &Map) -> bool {
    match goal {
        Goal::GatherResource(resource_name) => {
            if let Some(resource_def) = map.resources.iter().find(|r| &r.name == resource_name) {
                if resource_def.huntable {
                    return true;
                }
            }
            known_resources
                .0
                .get(resource_name)
                .map_or(false, |s| !s.is_empty())
        }
        _ => true,
    }
}

/// Creates a plan (a sequence of sub-goals) to achieve a given high-level goal.
pub fn plan_goal(
    brain: &BrainComponent,
    inventory: &Inventory,
    known_resources: &KnownResources,
    goal: &Goal,
) -> Result<Vec<Goal>, SimulationError> {
    let mut plan = Vec::new();
    match goal {
        Goal::CraftItem(item_name) | Goal::Build(item_name) => {
            plan.extend(plan_crafting_or_building(
                item_name,
                brain,
                inventory,
                known_resources,
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

/// Helper function to plan the gathering of resources for crafting or building.
fn plan_crafting_or_building(
    item_name: &str,
    brain: &BrainComponent,
    inventory: &Inventory,
    known_resources: &KnownResources,
) -> Vec<Goal> {
    let mut plan = Vec::new();
    let required = brain.recipe_manager.get_required_resources(item_name, 1);
    plan.extend(plan_resource_gathering(
        inventory,
        known_resources,
        &required,
    ));
    plan
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
