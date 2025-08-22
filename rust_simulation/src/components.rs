use crate::brain::{Goal, HighLevelState};
use crate::recipes::RecipeManager;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub mod intents;
pub mod path;
pub mod ai;

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
    pub current_goal: Option<Goal>,
    pub goal_stack: Vec<Goal>,
    pub goal_commitment_ticks: u32,
    pub prev_state: Option<HighLevelState>,
    pub prev_goal: Option<Goal>,
    pub home_base: Option<Position>,

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
            current_goal: None,
            goal_stack: Vec::new(),
            goal_commitment_ticks: 0,
            prev_state: None,
            prev_goal: None,
            home_base: None,
            goals,
            recipe_manager,
            learning_rate,
            discount_factor,
            epsilon,
        }
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
