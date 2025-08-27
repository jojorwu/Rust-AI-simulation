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

// Type alias to simplify the query type
type GoalSelectionQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut BrainComponent,
        &'static Health,
        &'static Hunger,
        &'static Inventory,
        &'static KnownResources,
        &'static PlayerMemories,
        &'static GoalQTable,
    ),
>;

// Struct to hold the arguments for choose_goal, solving the `too_many_arguments` lint.
struct ChooseGoalArgs<'a, R: Rng + ?Sized> {
    state: &'a HighLevelState,
    brain: &'a BrainComponent,
    inventory: &'a Inventory,
    known_resources: &'a KnownResources,
    goal_q_table: &'a GoalQTable,
    is_day: bool,
    config: &'a Config,
    map: &'a Map,
    rng: &'a mut R,
}

pub fn goal_selection_system(
    mut query: GoalSelectionQuery,
    is_day: Res<IsDay>,
    config: Res<Config>,
    map: Res<Map>,
) {
    query.par_iter_mut().for_each(
        |(entity, mut brain, health, hunger, inventory, known_resources, player_memories, goal_q_table)| {
            if brain.current_goal.is_none() && brain.goal_commitment_ticks == 0 {
                let high_level_state =
                    get_high_level_state(health, hunger, inventory, player_memories, is_day.0);

                let mut rng = rand::thread_rng();
                let mut args = ChooseGoalArgs {
                    state: &high_level_state,
                    brain: &brain,
                    inventory,
                    known_resources,
                    goal_q_table,
                    is_day: is_day.0,
                    config: &config,
                    map: &map,
                    rng: &mut rng,
                };

                if let Ok(new_high_level_goal) = choose_goal(&mut args) {
                    brain.current_goal = Some(new_high_level_goal);
                    if let Some(goal) = &brain.current_goal {
                        info!("Entity {entity:?} selected new goal: {goal:?}");
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
fn choose_goal<R: Rng + ?Sized>(args: &mut ChooseGoalArgs<R>) -> Result<Goal, SimulationError> {
    const FLEE_HEALTH_THRESHOLD: u32 = 25;
    if args.state.health_level < FLEE_HEALTH_THRESHOLD {
        return Ok(Goal::Flee);
    }

    const HUNGER_THRESHOLD: u32 = 50;
    if args.state.hunger_level < HUNGER_THRESHOLD {
        if args.inventory.has_item("meat", 1) {
            return Ok(Goal::EatFood("meat".to_string()));
        } else {
            return Ok(Goal::GatherResource("pig".to_string()));
        }
    }

    let valid_goals: Vec<_> = args
        .brain
        .goals
        .iter()
        .filter(|g| is_goal_valid(g, args.known_resources, args.map))
        .cloned()
        .collect();
    if valid_goals.is_empty() {
        return Ok(Goal::Flee);
    }

    if args.rng.random::<f64>() < args.brain.epsilon {
        let index = args.rng.random_range(0..valid_goals.len());
        return Ok(valid_goals[index].clone());
    }

    if let Some(q_values) = args.goal_q_table.0.get(args.state) {
        q_values
            .iter()
            .filter(|(g, _)| is_goal_valid(g, args.known_resources, args.map))
            .map(|(goal, q_value)| {
                let effective_q_value = if !args.is_day {
                    if let Goal::Build(_) = goal {
                        *q_value + args.config.ai.goals.build_bonus
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
                let index = args.rng.random_range(0..valid_goals.len());
                Ok(valid_goals[index].clone())
            })
    } else {
        let index = args.rng.random_range(0..valid_goals.len());
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
                .is_some_and(|s| !s.is_empty())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipes::RecipeManager;
    use std::sync::Arc;

    #[test]
    fn test_plan_goal_craft_item() {
        let recipe_manager = Arc::new(
            RecipeManager::new("data/recipes.json").expect("Failed to create recipe manager"),
        );
        let brain = BrainComponent::new(Arc::clone(&recipe_manager), 0.1, 0.9, 1.0);
        let inventory = Inventory::new();
        let known_resources = KnownResources(HashMap::new());
        let goal = Goal::CraftItem("stone_axe".to_string());

        let plan = plan_goal(&brain, &inventory, &known_resources, &goal)
            .expect("Planning should succeed");

        assert_eq!(plan.len(), 5);
        assert!(plan.contains(&Goal::Explore));
        assert!(plan.contains(&Goal::GatherResource("wood".to_string())));
        assert!(plan.contains(&Goal::GatherResource("stone".to_string())));
        assert_eq!(plan[4], Goal::CraftItem("stone_axe".to_string()));
    }

    #[test]
    fn test_plan_goal_build_chest() {
        let recipe_manager = Arc::new(
            RecipeManager::new("data/recipes.json").expect("Failed to create recipe manager"),
        );
        let brain = BrainComponent::new(Arc::clone(&recipe_manager), 0.1, 0.9, 1.0);
        let inventory = Inventory::new();
        let known_resources = KnownResources(HashMap::new());
        let goal = Goal::Build("chest".to_string());

        let plan = plan_goal(&brain, &inventory, &known_resources, &goal)
            .expect("Planning should succeed");

        assert_eq!(plan.len(), 3);
        assert!(plan.contains(&Goal::Explore));
        assert!(plan.contains(&Goal::GatherResource("wood".to_string())));
        assert_eq!(plan[2], Goal::Build("chest".to_string()));
    }

    #[test]
    fn test_choose_goal_flee_when_low_health() {
        let recipe_manager = Arc::new(
            RecipeManager::new("data/recipes.json").expect("Failed to create recipe manager"),
        );
        let brain = BrainComponent::new(Arc::clone(&recipe_manager), 0.1, 0.9, 1.0);
        let inventory = Inventory::new();
        let known_resources = KnownResources(HashMap::new());
        let goal_q_table = GoalQTable(HashMap::new());
        let config = Config::load("data/config.toml").expect("Failed to load config");
        let map = Map::new(10, 10, "data/biomes.json", "data/resources.json")
            .expect("Failed to create map");
        let mut rng = rand::thread_rng();

        let state = HighLevelState {
            inventory_summary: InventorySummary {
                wood: 0,
                stone: 0,
                iron_ore: 0,
                stone_axe: 0,
            },
            num_hostile_players: 0,
            health_level: 10, // Low health
            hunger_level: 100,
            is_night: false,
        };

        let mut args = ChooseGoalArgs {
            state: &state,
            brain: &brain,
            inventory: &inventory,
            known_resources: &known_resources,
            goal_q_table: &goal_q_table,
            is_day: true,
            config: &config,
            map: &map,
            rng: &mut rng,
        };

        let goal = choose_goal(&mut args).expect("Choose goal should succeed");
        assert_eq!(goal, Goal::Flee);
    }
}
