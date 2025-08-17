const INVENTORY_SLOTS: usize = 6;

use serde::{Serialize, Deserialize};
use super::item::ItemRegistry;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Slot {
    pub item: String,
    pub quantity: u32,
    pub instance_id: Option<u32>,
    pub durability: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub _held_item: Option<String>,
    pub inventory: Vec<Option<Slot>>,
}

use std::collections::HashMap;

impl Player {
    pub fn new(_id: u32) -> Self {
        Player {
            _held_item: None,
            inventory: vec![None; INVENTORY_SLOTS],
        }
    }

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

    pub fn reset(&mut self) {
        self._held_item = None;
        self.inventory = vec![None; INVENTORY_SLOTS];
    }
}

use crate::ecs::Component;

impl Component for Player {}
