use crate::brain::{DiscretizedLevel, Goal, HighLevelState, InventorySummary};
use crate::ItemRegistryResource;
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
use rand::prelude::*;

// Type alias to simplify the query type
use crate::components::Position;
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
        &'static Position,
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
    position: &'a Position,
    player_memories: &'a PlayerMemories,
    rng: &'a mut R,
}

pub fn goal_selection_system(
    mut query: GoalSelectionQuery,
    is_day: Res<IsDay>,
    config: Res<Config>,
    map: Res<Map>,
    item_registry: Res<crate::ItemRegistryResource>,
) {
    query.par_iter_mut().for_each(
        |(entity, mut brain, health, hunger, inventory, known_resources, player_memories, goal_q_table, position)| {
            if brain.current_goal.is_none() && brain.goal_commitment_ticks == 0 {
                let high_level_state =
                    get_high_level_state(health, hunger, inventory, player_memories, is_day.0);

                let mut rng = rand::rng();
                let mut args = ChooseGoalArgs {
                    state: &high_level_state,
                    brain: &brain,
                    inventory,
                    known_resources,
                    goal_q_table,
                    is_day: is_day.0,
                    config: &config,
                    map: &map,
                    position,
                    player_memories,
                    rng: &mut rng,
                };

                if let Ok(new_high_level_goal) = choose_goal(&mut args, &item_registry) {
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

use std::collections::HashSet;

struct PlannerArgs<'a> {
    brain: &'a BrainComponent,
    inventory: &'a Inventory,
    known_resources: &'a KnownResources,
    item_registry: &'a ItemRegistryResource,
    map: &'a Map,
    processed_goals: &'a mut HashSet<Goal>,
}

pub fn goal_planning_system(
    mut query: Query<(Entity, &mut BrainComponent, &Inventory, &KnownResources)>,
    item_registry: Res<ItemRegistryResource>,
    map: Res<Map>,
) {
    for (_entity, mut brain, inventory, known_resources) in query.iter_mut() {
        if let Some(goal) = &brain.current_goal {
            if brain.goal_stack.is_empty() {
                let mut planner_args = PlannerArgs {
                    brain: &brain,
                    inventory,
                    known_resources,
                    item_registry: &item_registry,
                    map: &map,
                    processed_goals: &mut HashSet::new(),
                };
                if let Ok(mut plan) = plan_goal(&mut planner_args, goal) {
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
                Goal::GatherResource(res, amount) => {
                    c.entity(entity)
                        .insert(IntendsToGather(res.clone(), *amount));
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

    let inventory_summary = InventorySummary::from(inventory);

    let health_percent = (health.current as f32 / health.max as f32) * 100.0;
    let health_level = if health_percent < 34.0 {
        DiscretizedLevel::Low
    } else if health_percent < 67.0 {
        DiscretizedLevel::Medium
    } else {
        DiscretizedLevel::High
    };

    let hunger_percent = (hunger.current / hunger.max) * 100.0;
    let hunger_level = if hunger_percent < 34.0 {
        DiscretizedLevel::Low
    } else if hunger_percent < 67.0 {
        DiscretizedLevel::Medium
    } else {
        DiscretizedLevel::High
    };

    HighLevelState {
        inventory_summary,
        num_hostile_players,
        health_level,
        hunger_level,
        is_night: !is_day,
    }
}

/// Chooses a high-level goal for the agent based on the current state.
/// This function prioritizes critical needs (fleeing, eating) before consulting the Q-table.
fn choose_goal<R: Rng + ?Sized>(
    args: &mut ChooseGoalArgs<R>,
    item_registry: &crate::ItemRegistryResource,
) -> Result<Goal, SimulationError> {
    if let Some(critical_goal) = handle_critical_needs(args, item_registry) {
        return Ok(critical_goal);
    }

    choose_q_learning_goal(args)
}

/// Handles immediate, critical needs like fleeing from low health or eating when starving.
/// Returns Some(Goal) if a critical action is necessary, otherwise None.
fn handle_critical_needs<R: Rng + ?Sized>(
    args: &ChooseGoalArgs<R>,
    item_registry: &crate::ItemRegistryResource,
) -> Option<Goal> {
    if args.state.health_level == DiscretizedLevel::Low {
        return Some(Goal::Flee);
    }

    if args.state.hunger_level == DiscretizedLevel::Low {
        // 1. Find any food in inventory
        let food_in_inventory = args.inventory.items.keys().find(|item_name| {
            item_registry
                .0
                .get_item(item_name)
                .is_some_and(|item| item.is_food)
        });

        if let Some(food_name) = food_in_inventory {
            return Some(Goal::EatFood(food_name.clone()));
        } else {
            // 2. If no food, find a known resource that is a food source.
            // This is a simplification. A better AI would check the loot table of resources.
            // For now, we'll check for any huntable resource.
            if let Some(huntable) = args.map.resources.iter().find(|r| r.huntable) {
                return Some(Goal::GatherResource(huntable.name.clone(), 1));
            }
        }
    }

    None
}

/// Chooses a goal based on the Q-learning model (exploration vs. exploitation).
fn choose_q_learning_goal<R: Rng + ?Sized>(
    args: &mut ChooseGoalArgs<R>,
) -> Result<Goal, SimulationError> {
    let valid_goals: Vec<_> = args
        .brain
        .goals
        .iter()
        .filter(|g| is_goal_valid(g, args.known_resources, args.map))
        .cloned()
        .collect();
    if valid_goals.is_empty() {
        // If no goals are valid, fleeing is a safe default.
        return Ok(Goal::Flee);
    }

    // Epsilon-greedy strategy for exploration vs. exploitation.
    // With a probability of epsilon, we choose a random valid action (explore).
    if args.rng.random::<f64>() < args.brain.epsilon {
        // Exploration: Choose a random valid goal.
        let index = args.rng.random_range(0..valid_goals.len());
        return Ok(valid_goals[index].clone());
    }

    // Exploitation: choose the best-known goal from the Q-table.
    if let Some(q_values) = args.goal_q_table.0.get(args.state) {
        q_values
            .iter()
            .filter(|(g, _)| is_goal_valid(g, args.known_resources, args.map))
            .map(|(goal, q_value)| {
                let mut effective_q_value = *q_value;

                // Apply situational bonuses/penalties
                // 1. Bonus for building at night
                if !args.is_day {
                    if let Goal::Build(_) = goal {
                        effective_q_value += args.config.ai.goals.build_bonus;
                    }
                }

                // 2. Proximity bonus for gathering resources
                if let Goal::GatherResource(resource, _amount) = goal {
                    if let Some(positions) = args.known_resources.0.get(resource) {
                        if let Some(closest_pos) = positions.iter().min_by(|a, b| {
                            a.distance_squared(args.position)
                                .total_cmp(&b.distance_squared(args.position))
                        }) {
                            let dist = closest_pos.distance(args.position);
                            // Simple bonus: higher for closer resources. Capped at a max bonus.
                            let proximity_bonus = (1.0 / (dist + 1.0)) * 5.0; // Example bonus calculation
                            effective_q_value += proximity_bonus as f64;
                        }
                    }
                }

                // 3. Safety penalty if hostile players are nearby
                if args.player_memories.0.values().any(|m| m.relationship == crate::brain::RelationshipStatus::Hostile) {
                    // Simple penalty if any hostile is known, could be refined by distance.
                    effective_q_value -= 20.0; // Large penalty
                }

                (goal, effective_q_value)
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(goal, _)| goal.clone())
            .map(Ok)
            .unwrap_or_else(|| {
                // Fallback to random choice if max_by is empty (e.g., no valid goals in q-table)
                let index = args.rng.random_range(0..valid_goals.len());
                Ok(valid_goals[index].clone())
            })
    } else {
        // If there's no entry for the current state in the Q-table, choose a random valid goal.
        let index = args.rng.random_range(0..valid_goals.len());
        Ok(valid_goals[index].clone())
    }
}

/// Checks if a goal is currently valid.
fn is_goal_valid(goal: &Goal, known_resources: &KnownResources, map: &Map) -> bool {
    match goal {
        Goal::GatherResource(resource_name, _amount) => {
            if let Some(resource_def) = map.resources.iter().find(|r| r.name == *resource_name) {
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
/// This is a simple hierarchical planner that can now handle nested dependencies.
fn plan_goal(args: &mut PlannerArgs, goal: &Goal) -> Result<Vec<Goal>, SimulationError> {
    // Prevent infinite recursion if there's a circular dependency.
    if !args.processed_goals.insert(goal.clone()) {
        return Ok(Vec::new());
    }

    let mut plan = Vec::new();
    match goal {
        Goal::CraftItem(item_name) | Goal::Build(item_name) => {
            // For crafting or building, first plan to gather the required resources.
            plan.extend(plan_crafting_or_building(args, item_name)?);
            // Then, add the final craft/build goal itself.
            plan.push(goal.clone());
        }
        Goal::Stockpile(resource) => {
            let has_enough = args.inventory.has_item(resource, 1);
            if !has_enough {
                plan.extend(plan_goal(args, &Goal::GatherResource(resource.clone(), 1))?);
            }
            plan.push(goal.clone());
        }
        Goal::GatherResource(resource_name, amount) => {
            // This is the core of the tool-aware and dependency logic.
            // First, plan to acquire the necessary tool, if any.
            plan.extend(plan_tool_for_resource(args, resource_name)?);
            // Then, plan to find the resource if its location is unknown.
            if !args.known_resources.0.contains_key(resource_name) {
                plan.push(Goal::Explore);
            }
            // Finally, add the goal to gather the resource itself.
            plan.push(Goal::GatherResource(resource_name.clone(), *amount));
        }
        _ => {
            // For simple goals, the plan is just the goal itself.
            plan.push(goal.clone());
        }
    }
    Ok(plan)
}

/// Helper function to create a plan for crafting dependencies.
fn plan_crafting_or_building(
    args: &mut PlannerArgs,
    item_name: &str,
) -> Result<Vec<Goal>, SimulationError> {
    let mut plan = Vec::new();
    let required = args
        .brain
        .recipe_manager
        .get_required_resources(item_name, 1)?;

    for (resource_name, &required_amount) in &required {
        let has_amount = args.inventory.get_quantity(resource_name);
        if has_amount < required_amount {
            // If the required resource is itself a craftable item, recurse.
            if args
                .brain
                .recipe_manager
                .recipes
                .contains_key(resource_name)
            {
                plan.extend(plan_goal(
                    args,
                    &Goal::CraftItem(resource_name.clone()),
                )?);
            } else {
                // Otherwise, it's a raw resource that needs to be gathered.
                let amount_needed = required_amount - has_amount;
                plan.extend(plan_goal(
                    args,
                    &Goal::GatherResource(resource_name.clone(), amount_needed),
                )?);
            }
        }
    }
    Ok(plan)
}

/// Helper function to create a plan to acquire a necessary tool for a resource.
fn plan_tool_for_resource(
    args: &mut PlannerArgs,
    resource_name: &str,
) -> Result<Vec<Goal>, SimulationError> {
    if let Some(resource_def) = args.map.resources.iter().find(|r| &r.name == resource_name) {
        if let Some(tool_category) = &resource_def.required_tool {
            // Check if the agent has a tool of the required category.
            let has_tool = args.inventory.items.keys().any(|item_name| {
                args.item_registry
                    .0
                    .get_item(item_name)
                    .is_some_and(|item| item.category.as_deref() == Some(tool_category))
            });

            if !has_tool {
                // Find all craftable tools of the required category.
                let mut candidate_tools: Vec<_> = args
                    .brain
                    .recipe_manager
                    .recipes
                    .keys()
                    .filter_map(|recipe_name| {
                        let item = args.item_registry.0.get_item(recipe_name)?;
                        if item.category.as_deref() == Some(tool_category) {
                            Some((recipe_name.clone(), item.tier))
                        } else {
                            None
                        }
                    })
                    .collect();

                // Sort tools by tier, highest first.
                candidate_tools.sort_by_key(|&(_, tier)| std::cmp::Reverse(tier));

                // Try to plan for the best tool possible.
                for (tool_name, _) in candidate_tools {
                    // Create a fresh set of processed goals for this sub-plan attempt.
                    let mut sub_planner_args = PlannerArgs {
                        processed_goals: &mut HashSet::new(),
                        ..*args
                    };
                    if let Ok(tool_plan) =
                        plan_goal(&mut sub_planner_args, &Goal::CraftItem(tool_name))
                    {
                        // If planning for this tool is successful, adopt this plan and stop.
                        return Ok(tool_plan);
                    }
                }
            }
        }
    }
    // Return an empty plan if no tool is needed, agent has a tool, or no tool can be crafted.
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipes::RecipeManager;
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_plan_goal_craft_item_with_recursive_planning() {
        let recipe_manager = Arc::new(
            RecipeManager::new("data/recipes.json").expect("Failed to create recipe manager"),
        );
        let item_registry = Arc::new(crate::item::ItemRegistry::new("data/items.json").unwrap());
        let map = Map::new(10, 10, "data/biomes.json", "data/resources.json").unwrap();
        let brain = BrainComponent::new(Arc::clone(&recipe_manager), 0.1, 0.9, 1.0);
        let inventory = Inventory::new();
        let known_resources = KnownResources(HashMap::new());
        let goal = Goal::CraftItem("stone_axe".to_string());

        let mut planner_args = PlannerArgs {
            brain: &brain,
            inventory: &inventory,
            known_resources: &known_resources,
            item_registry: &crate::ItemRegistryResource(item_registry),
            map: &map,
            processed_goals: &mut HashSet::new(),
        };

        let plan = plan_goal(&mut planner_args, &goal).expect("Planning should succeed");

        // The plan should contain all the necessary sub-goals, though the order of gathering
        // might change due to HashMap iteration order.
        assert_eq!(plan.len(), 5);
        assert_eq!(plan.last(), Some(&Goal::CraftItem("stone_axe".to_string())));
        assert!(plan.contains(&Goal::GatherResource("stone".to_string(), 3)));
        assert!(plan.contains(&Goal::GatherResource("wood".to_string(), 2)));
        // The plan should contain at least one explore goal since no resources are known.
        assert!(plan.contains(&Goal::Explore));
    }

    #[test]
    fn test_plan_goal_gather_with_tool_dependency() {
        let recipe_manager = Arc::new(
            RecipeManager::new("data/recipes.json").expect("Failed to create recipe manager"),
        );
        let item_registry = Arc::new(crate::item::ItemRegistry::new("data/items.json").unwrap());
        let map = Map::new(10, 10, "data/biomes.json", "data/resources.json").unwrap();
        let brain = BrainComponent::new(Arc::clone(&recipe_manager), 0.1, 0.9, 1.0);
        // Give the agent enough resources to craft an iron axe
        let mut inventory = Inventory::new();
        inventory.add_item("wood", 10);
        inventory.add_item("iron_bars", 10);
        let known_resources = KnownResources(HashMap::new());
        let goal = Goal::GatherResource("tree".to_string(), 1); // Trees require an axe

        let mut planner_args = PlannerArgs {
            brain: &brain,
            inventory: &inventory,
            known_resources: &known_resources,
            item_registry: &crate::ItemRegistryResource(item_registry),
            map: &map,
            processed_goals: &mut HashSet::new(),
        };

        let plan = plan_goal(&mut planner_args, &goal).expect("Planning should succeed");

        // The AI should be smart enough to plan to craft the BEST tool it can afford.
        // In this case, it has the resources for an iron_axe, so it should prefer that over a stone_axe.
        assert!(
            plan.contains(&Goal::CraftItem("iron_axe".to_string())),
            "Plan should include crafting an iron axe"
        );
        assert!(
            !plan.contains(&Goal::CraftItem("stone_axe".to_string())),
            "Plan should NOT include crafting a stone axe if an iron one is possible"
        );
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
        let mut rng = rand::rng();

        let state = HighLevelState {
            inventory_summary: InventorySummary {
                items: BTreeMap::new(),
            },
            num_hostile_players: 0,
            health_level: DiscretizedLevel::Low,
            hunger_level: DiscretizedLevel::High,
            is_night: false,
        };

        let player_memories = PlayerMemories(HashMap::new());
        let position = Position { x: 0, y: 0 };
        let mut args = ChooseGoalArgs {
            state: &state,
            brain: &brain,
            inventory: &inventory,
            known_resources: &known_resources,
            goal_q_table: &goal_q_table,
            is_day: true,
            config: &config,
            map: &map,
            player_memories: &player_memories,
            position: &position,
            rng: &mut rng,
        };

        let item_registry =
            crate::ItemRegistryResource(Arc::new(crate::item::ItemRegistry::new("data/items.json").unwrap()));
        let goal = choose_goal(&mut args, &item_registry).expect("Choose goal should succeed");
        assert_eq!(goal, Goal::Flee);
    }
}
