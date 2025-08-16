const INVENTORY_SLOTS: usize = 6;

use serde::{Serialize, Deserialize};
use super::item::ItemRegistry;
use super::entity::Entity;
use super::actions::Action;
use super::game::Game;
use super::errors::SimulationError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Slot {
    pub item: String,
    pub quantity: u32,
    pub instance_id: Option<u32>,
    pub durability: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub health: i32,
    pub held_item: Option<String>,
    pub inventory: Vec<Option<Slot>>,
}

use std::collections::HashMap;
use std::any::Any;

impl Player {
    pub fn new(id: u32, x: u32, y: u32) -> Self {
        Player {
            id,
            x,
            y,
            health: 100,
            held_item: None,
            inventory: vec![None; INVENTORY_SLOTS],
        }
    }

    pub fn reset(&mut self) {
        self.health = 100;
        self.inventory = vec![None; INVENTORY_SLOTS];
        self.held_item = None;
    }

    // --- Inventory Helper Methods ---

    fn find_item_slot(&self, item_name: &str) -> Option<usize> {
        self.inventory.iter().position(|slot| {
            if let Some(s) = slot {
                s.item == item_name
            } else {
                false
            }
        })
    }

    fn find_empty_slot(&self) -> Option<usize> {
        self.inventory.iter().position(|slot| slot.is_none())
    }

    pub fn has_lock(&self) -> bool {
        self.inventory.iter().any(|s| s.as_ref().map_or(false, |slot| slot.item == "lock"))
    }

    pub fn find_and_remove_lock(&mut self) -> Option<u32> {
        let lock_slot_index = self.inventory.iter().position(|s| s.as_ref().map_or(false, |slot| slot.item == "lock"));
        if let Some(index) = lock_slot_index {
            let lock_id = self.inventory[index].as_ref().unwrap().instance_id;
            self.inventory[index] = None;
            return lock_id;
        }
        None
    }

    pub fn has_key(&self, key_id: u32) -> bool {
        self.inventory.iter().any(|s| s.as_ref().map_or(false, |slot| slot.item == "key" && slot.instance_id == Some(key_id)))
    }

    pub fn get_total_quantity(&self, item_name: &str) -> u32 {
        self.inventory.iter().filter_map(|slot| {
            slot.as_ref().and_then(|s| {
                if s.item == item_name { Some(s.quantity) } else { None }
            })
        }).sum()
    }

    pub fn has_resources(&self, recipe: &HashMap<String, u32>) -> bool {
        for (resource, required_amount) in recipe {
            if self.get_total_quantity(resource) < *required_amount {
                return false;
            }
        }
        true
    }

    pub fn add_item(&mut self, item_name: &str, quantity: u32, instance_id: Option<u32>, item_registry: &ItemRegistry) -> bool {
        let item = item_registry.get_item(item_name);

        if let Some(item_def) = item {
            // Stack only if the item is stackable AND we are not adding a unique instance
            if item_def.stackable && instance_id.is_none() {
                if let Some(slot_index) = self.find_item_slot(item_name) {
                    if let Some(slot) = &mut self.inventory[slot_index] {
                        slot.quantity += quantity;
                        return true;
                    }
                }
            }
        }

        // If not stackable, or a unique instance, or no existing stack, find an empty slot
        if let Some(empty_slot_index) = self.find_empty_slot() {
            let initial_durability = if let Some(item_def) = item {
                if item_def.tool {
                    item_def.properties.as_ref().and_then(|p| p.get("max_durability").cloned())
                } else { None }
            } else { None };

            self.inventory[empty_slot_index] = Some(Slot {
                item: item_name.to_string(),
                quantity,
                instance_id,
                durability: initial_durability,
            });
            true
        } else {
            false // Inventory is full
        }
    }

    pub fn remove_resources(&mut self, recipe: &HashMap<String, u32>) -> bool {
        if !self.has_resources(recipe) {
            return false;
        }

        for (resource, &amount_to_remove) in recipe {
            let mut removed_so_far = 0;
            while removed_so_far < amount_to_remove {
                let amount_needed = amount_to_remove - removed_so_far;

                let slot_index = self.inventory.iter().position(|s| s.as_ref().map_or(false, |slot| &slot.item == resource && slot.quantity > 0));

                if let Some(i) = slot_index {
                    if let Some(slot) = &mut self.inventory[i] {
                        let can_remove = std::cmp::min(amount_needed, slot.quantity);
                        slot.quantity -= can_remove;
                        removed_so_far += can_remove;

                        if slot.quantity == 0 {
                            self.inventory[i] = None;
                        }
                    }
                } else {
                    return false;
                }
            }
        }
        true
    }

    pub fn move_player(&mut self, direction: &str, map: &super::map::Map, entities: &[Box<dyn Entity>]) -> bool {
        let (mut dx, mut dy) = (0, 0);
        match direction {
            "up" => dy = -1,
            "down" => dy = 1,
            "left" => dx = -1,
            "right" => dx = 1,
            _ => return false,
        }

        let new_x = (self.x as i32 + dx) as u32;
        let new_y = (self.y as i32 + dy) as u32;

        if new_x < map.width && new_y < map.height {
            let target_tile = &map.grid[new_y as usize][new_x as usize];
            let blocking_tiles = ['W', '#', 'D', 'L'];
            if !blocking_tiles.contains(&target_tile.tile_type) {
                // Check for other entities
                for entity in entities {
                    if entity.get_id() != self.id {
                        let (ex, ey) = entity.get_position();
                        if ex == new_x && ey == new_y {
                            return false; // Another entity is in the way
                        }
                    }
                }

                self.x = new_x;
                self.y = new_y;
                return true;
            }
        }
        false
    }
}

impl Entity for Player {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_position(&self) -> (u32, u32) {
        (self.x, self.y)
    }

    fn get_health(&self) -> i32 {
        self.health
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }

    fn update(&mut self, _game: &Game) -> Result<Option<Action>, SimulationError> {
        // Player logic is handled in the game loop for now
        Ok(None)
    }
}
