const INVENTORY_SLOTS: usize = 6;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Slot {
    pub item: String,
    pub quantity: u32,
}

#[derive(Debug)]
pub struct Player {
    pub x: u32,
    pub y: u32,
    pub held_item: Option<String>,
    pub inventory: Vec<Option<Slot>>,
}

use std::collections::HashMap;

impl Player {
    pub fn new(x: u32, y: u32) -> Self {
        Player {
            x,
            y,
            held_item: None,
            inventory: vec![None; INVENTORY_SLOTS],
        }
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

    pub fn add_item(&mut self, item_name: &str, quantity: u32) -> bool {
        let stackable_items = ["wood", "stone", "sulfur", "iron_ore", "iron_bars"];

        if stackable_items.contains(&item_name) {
            if let Some(slot_index) = self.find_item_slot(item_name) {
                if let Some(slot) = &mut self.inventory[slot_index] {
                    slot.quantity += quantity;
                    return true;
                }
            }
        }

        if let Some(empty_slot_index) = self.find_empty_slot() {
            self.inventory[empty_slot_index] = Some(Slot {
                item: item_name.to_string(),
                quantity,
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

    pub fn move_player(&mut self, direction: &str, map: &super::map::Map) -> bool {
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

        if new_x < map.width && new_y < map.height && map.grid[new_y as usize][new_x as usize] != 'W' {
            self.x = new_x;
            self.y = new_y;
            true
        } else {
            false
        }
    }
}
