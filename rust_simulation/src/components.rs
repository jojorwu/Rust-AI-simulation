use crate::brain::{Goal, HighLevelState, InventorySummary, MemoryTile, PlayerMemory};
use crate::errors::SimulationError;
use crate::recipes::RecipeManager;
use bevy_ecs::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub mod intents;

#[derive(Component, Debug, Clone, Copy, Eq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Hash for Position {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity {
    pub dx: i32,
    pub dy: i32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WantsToGather {
    pub target: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToCraft {
    pub item_name: String,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToBuild {
    pub structure_name: String,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Chest {
    pub inventory: Inventory,
}

#[derive(Component, Clone, Debug)]
pub struct WantsToStoreItem {
    pub item_name: String,
    pub quantity: u32,
    pub target_chest: Entity,
}

#[derive(Component, Clone)]
pub struct BrainComponent {
    pub mental_map: Vec<Vec<Option<MemoryTile>>>,
    pub known_resources: HashMap<String, HashSet<Position>>,
    pub player_memories: HashMap<Entity, PlayerMemory>,
    pub current_goal: Option<Goal>,
    pub goal_stack: Vec<Goal>,
    pub current_path: Option<Vec<(u32, u32)>>,
    pub goal_commitment_ticks: u32,
    pub prev_state: Option<HighLevelState>,
    pub prev_goal: Option<Goal>,
    pub home_base: Option<Position>,
    pub goal_q_table: HashMap<HighLevelState, HashMap<Goal, f64>>,
    pub exploration_frontier: VecDeque<Position>,

    // Fields from Brain
    pub goals: Vec<Goal>,
    pub recipe_manager: Arc<RecipeManager>,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub epsilon: f64,
}

impl BrainComponent {
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
        BrainComponent {
            mental_map: vec![
                vec![None; crate::config::WIDTH as usize];
                crate::config::HEIGHT as usize
            ],
            known_resources: HashMap::new(),
            player_memories: HashMap::new(),
            current_goal: None,
            goal_stack: Vec::new(),
            current_path: None,
            goal_commitment_ticks: 0,
            prev_state: None,
            prev_goal: None,
            home_base: None,
            goal_q_table: HashMap::new(),
            exploration_frontier: VecDeque::new(),
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
        is_day: bool,
    ) -> HighLevelState {
        let num_hostile_players = self
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
        state: &HighLevelState,
    ) -> Result<Goal, SimulationError> {
        // TODO: Improve this. The health level should ideally be a percentage.
        const FLEE_HEALTH_THRESHOLD: u32 = 25;
        if state.health_level < FLEE_HEALTH_THRESHOLD {
            return Ok(Goal::Flee);
        }

        let valid_goals: Vec<_> = self
            .goals
            .iter()
            .filter(|g| self.is_goal_valid(g))
            .cloned()
            .collect();
        if valid_goals.is_empty() {
            return Ok(Goal::Flee);
        }

        let mut rng = rand::rng();
        if rng.random::<f64>() < self.epsilon {
            let index = rng.random_range(0..valid_goals.len());
            return Ok(valid_goals[index].clone());
        }

        if let Some(q_values) = self.goal_q_table.get(state) {
            q_values
                .iter()
                .filter(|(g, _)| self.is_goal_valid(g))
                .map(|(goal, q_value)| {
                    let effective_q_value = if state.is_night {
                        if let Goal::Build(_) = goal {
                            *q_value + crate::config::BUILD_GOAL_BONUS
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
    fn is_goal_valid(&self, goal: &Goal) -> bool {
        match goal {
            Goal::GatherResource(resource_name) => self
                .known_resources
                .get(resource_name)
                .is_some_and(|p| !p.is_empty()),
            _ => true,
        }
    }

    /// Creates a plan (a sequence of sub-goals) to achieve a given high-level goal.
    pub fn plan_goal(
        &self,
        inventory: &Inventory,
        goal: &Goal,
    ) -> Result<Vec<Goal>, SimulationError> {
        let mut plan = Vec::new();
        match goal {
            Goal::CraftItem(item_name) => {
                let required = self.recipe_manager.get_required_resources(item_name, 1);
                plan.extend(self.plan_resource_gathering(inventory, &required));
                plan.push(goal.clone());
            }
            Goal::Build(structure_name) => {
                let required = self
                    .recipe_manager
                    .get_required_resources(structure_name, 1);
                plan.extend(self.plan_resource_gathering(inventory, &required));
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
        inventory: &Inventory,
        required: &HashMap<String, u32>,
    ) -> Vec<Goal> {
        let mut plan = Vec::new();
        for (resource, &required_amount) in required {
            let has_enough = inventory.get_quantity(resource) >= required_amount;
            if !has_enough {
                if !self.known_resources.contains_key(resource) {
                    plan.push(Goal::Explore);
                }
                plan.push(Goal::GatherResource(resource.clone()));
            }
        }
        plan
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WantsToAttack {
    pub target: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WantsToPickup {}

#[derive(Component, Debug, Clone)]
pub struct Resource {
    pub name: String,
    pub quantity: u32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component, Debug, Clone)]
pub struct DroppedItem {
    pub item_name: String,
    pub quantity: u32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: HashMap<String, u32>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            items: HashMap::new(),
        }
    }

    pub fn add_item(&mut self, item_name: &str, quantity: u32) {
        *self.items.entry(item_name.to_string()).or_insert(0) += quantity;
    }

    pub fn remove_item(&mut self, item_name: &str, quantity: u32) -> bool {
        if let Some(count) = self.items.get_mut(item_name) {
            if *count >= quantity {
                *count -= quantity;
                if *count == 0 {
                    self.items.remove(item_name);
                }
                return true;
            }
        }
        false
    }

    pub fn has_item(&self, item_name: &str, quantity: u32) -> bool {
        self.items
            .get(item_name)
            .is_some_and(|&count| count >= quantity)
    }

    pub fn get_quantity(&self, item_name: &str) -> u32 {
        *self.items.get(item_name).unwrap_or(&0)
    }

    pub fn has_resources(&self, recipe: &HashMap<String, u32>) -> bool {
        for (resource, &required_amount) in recipe {
            if self.get_quantity(resource) < required_amount {
                return false;
            }
        }
        true
    }

    pub fn remove_resources(&mut self, recipe: &HashMap<String, u32>) -> bool {
        if !self.has_resources(recipe) {
            return false;
        }
        for (resource, &amount_to_remove) in recipe {
            self.remove_item(resource, amount_to_remove);
        }
        true
    }
}
