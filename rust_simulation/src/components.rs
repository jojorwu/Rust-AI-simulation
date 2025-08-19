use crate::ecs::{Component, Entity};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Eq)]
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

impl Component for Position {}

#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub dx: i32,
    pub dy: i32,
}

impl Component for Velocity {}

#[derive(Debug, Clone, Copy)]
pub struct WantsToGather {
    pub target: Entity,
}

impl Component for WantsToGather {}

#[derive(Debug, Clone)]
pub struct WantsToCraft {
    pub item_name: String,
}

impl Component for WantsToCraft {}

#[derive(Debug, Clone)]
pub struct WantsToBuild {
    pub structure_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chest {
    pub inventory: Inventory,
}

#[derive(Clone, Debug)]
pub struct WantsToStoreItem {
    pub item_name: String,
    pub quantity: u32,
    pub target_chest: Entity,
}

impl Component for WantsToBuild {}
impl Component for Chest {}
impl Component for WantsToStoreItem {}

#[derive(Debug, Clone, Copy)]
pub struct WantsToAttack {
    pub target: Entity,
}

impl Component for WantsToAttack {}

#[derive(Debug, Clone, Copy)]
pub struct WantsToPickup {}

impl Component for WantsToPickup {}

#[derive(Debug, Clone)]
pub struct Resource {
    pub name: String,
    pub quantity: u32,
}

impl Component for Resource {}

#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Component for Health {}

#[derive(Debug, Clone)]
pub struct DroppedItem {
    pub item_name: String,
    pub quantity: u32,
}

impl Component for DroppedItem {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: HashMap<String, u32>,
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
        self.items.get(item_name).map_or(false, |&count| count >= quantity)
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

impl Component for Inventory {}
