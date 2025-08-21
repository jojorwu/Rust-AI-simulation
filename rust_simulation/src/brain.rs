//! This module contains the core AI logic for the simulation agents.
//!
//! The main components are:
//! - `Goal`: An enum representing the high-level objectives an agent can have.
//! - `Brain`: A struct that encapsulates the decision-making logic for an agent.
//! - `BrainAction`: An enum representing the concrete actions an agent can take.
//!
//! The `Brain` uses a Q-learning-based approach to decide on a `Goal`, and then
//! uses a planner to break that goal down into a series of actions.

use super::config::BUILD_GOAL_BONUS;
use bevy_ecs::prelude::*;
use super::errors::SimulationError;
use super::recipes::RecipeManager;
use crate::components::{
    BrainComponent, Health, Inventory, Velocity, WantsToGather, WantsToCraft, WantsToBuild,
    WantsToAttack, WantsToStoreItem,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// A tile as remembered by the agent.
#[derive(Debug, Clone)]
pub struct MemoryTile {
    pub tile: super::map::Tile,
}

/// The relationship status between agents.
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipStatus {
    /// The other agent is considered hostile.
    Hostile,
}

/// A memory of another player.
#[derive(Debug, Clone)]
pub struct PlayerMemory {
    pub relationship: RelationshipStatus,
}

/// Represents the high-level goals that an agent can have.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Goal {
    /// Gather a specific resource.
    GatherResource(String),
    /// Craft a specific item.
    CraftItem(String),
    /// Build a specific structure.
    Build(String),
    /// Attack a specific entity.
    Attack(Entity),
    /// Flee from a threat.
    Flee,
    /// Explore the map to find resources.
    Explore,
    /// Stockpile a resource in a chest.
    Stockpile(String),
}

/// Represents the concrete actions that an agent's brain can decide to take.
#[derive(Debug)]
pub enum BrainAction {
    /// Move in a specific direction.
    Move(Velocity),
    /// Gather a resource from a target entity.
    Gather(WantsToGather),
    /// Craft an item.
    Craft(WantsToCraft),
    /// Build a structure.
    Build(WantsToBuild),
    /// Attack a target entity.
    Attack(WantsToAttack),
    /// Store an item in a chest.
    Store(WantsToStoreItem),
}

/// A summary of the agent's inventory, used as part of the `HighLevelState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct InventorySummary {
    /// Whether the agent has any wood.
    pub has_wood: bool,
    /// Whether the agent has any stone.
    pub has_stone: bool,
    /// Whether the agent has any iron ore.
    pub has_iron_ore: bool,
    /// Whether the agent has a stone axe.
    pub has_stone_axe: bool,
}

/// Represents the high-level state of the agent and its environment.
/// This is used as the input to the Q-learning model for goal selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HighLevelState {
    /// A summary of the agent's inventory.
    pub inventory_summary: InventorySummary,
    /// The number of hostile players the agent is aware of.
    pub num_hostile_players: u32,
    /// The agent's current health level.
    pub health_level: u32,
    /// Whether it is currently night time.
    pub is_night: bool,
}

/// The `Brain` struct is a stateless logic processor for the AI.
/// It contains the core logic for decision-making, including goal selection,
/// planning, and action execution.
pub struct Brain {
    /// The list of possible goals the agent can choose from.
    pub goals: Vec<Goal>,
    /// A reference to the recipe manager for crafting information.
    pub recipe_manager: Arc<RecipeManager>,
    /// The learning rate for the Q-learning algorithm.
    pub learning_rate: f64,
    /// The discount factor for future rewards in the Q-learning algorithm.
    pub discount_factor: f64,
    /// The exploration factor (epsilon) for the epsilon-greedy policy.
    pub epsilon: f64,
}

impl Brain {
    /// Creates a new `Brain`.
    pub fn new(
        recipe_manager: Arc<RecipeManager>,
        learning_rate: f64,
        discount_factor: f64,
        epsilon: f64,
    ) -> Self {
        let goals = vec![
            Goal::GatherResource("wood".to_string()),
            Goal::GatherResource("stone".to_string()),
            Goal::CraftItem("stone_axe".to_string()),
            Goal::Build("foundation".to_string()),
            Goal::Stockpile("wood".to_string()),
        ];
        Brain {
            goals,
            recipe_manager,
            learning_rate,
            discount_factor,
            epsilon,
        }
    }

    /// Constructs the high-level state of an agent from its components.
    pub fn get_high_level_state(
        &self,
        health: &Health,
        inventory: &Inventory,
        brain_component: &BrainComponent,
        is_day: bool,
    ) -> HighLevelState {
        let num_hostile_players = brain_component
            .player_memories
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
    pub fn choose_goal(
        &self,
        brain_component: &BrainComponent,
        state: &HighLevelState,
    ) -> Result<Goal, SimulationError> {
        let valid_goals: Vec<_> = self
            .goals
            .iter()
            .filter(|g| self.is_goal_valid(brain_component, g))
            .cloned()
            .collect();
        if valid_goals.is_empty() {
            return Ok(Goal::Flee);
        }

        let mut rng = rand::thread_rng();

        if rng.random::<f64>() < self.epsilon {
            let index = rng.random_range(0..valid_goals.len());
            return Ok(valid_goals[index].clone());
        }

        if let Some(q_values) = brain_component.goal_q_table.get(state) {
            q_values
                .iter()
                .filter(|(g, _)| self.is_goal_valid(brain_component, g))
                .map(|(goal, q_value)| {
                    let effective_q_value = if state.is_night {
                        if let Goal::Build(_) = goal {
                            *q_value + BUILD_GOAL_BONUS
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
    fn is_goal_valid(&self, brain_component: &BrainComponent, goal: &Goal) -> bool {
        match goal {
            Goal::GatherResource(resource_name) => brain_component
                .known_resources
                .get(resource_name)
                .is_some_and(|p| !p.is_empty()),
            _ => true,
        }
    }

    /// Creates a plan (a sequence of sub-goals) to achieve a given high-level goal.
    pub fn plan_goal(
        &self,
        brain_component: &BrainComponent,
        inventory: &Inventory,
        goal: &Goal,
    ) -> Result<Vec<Goal>, SimulationError> {
        let mut plan = Vec::new();
        match goal {
            Goal::CraftItem(item_name) => {
                let required = self.recipe_manager.get_required_resources(item_name, 1);
                plan.extend(self.plan_resource_gathering(brain_component, inventory, &required));
                plan.push(goal.clone());
            }
            Goal::Build(structure_name) => {
                let required = self
                    .recipe_manager
                    .get_required_resources(structure_name, 1);
                plan.extend(self.plan_resource_gathering(brain_component, inventory, &required));
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
        &self,
        brain_component: &BrainComponent,
        inventory: &Inventory,
        required: &HashMap<String, u32>,
    ) -> Vec<Goal> {
        let mut plan = Vec::new();
        for (resource, &required_amount) in required {
            let has_enough = inventory.get_quantity(resource) >= required_amount;
            if !has_enough {
                if !brain_component.known_resources.contains_key(resource) {
                    plan.push(Goal::Explore);
                }
                plan.push(Goal::GatherResource(resource.clone()));
            }
        }
        plan
    }
}
