use crate::errors::SimulationError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    /// The unique name of the item.
    pub name: String,
    /// Whether this item can be stacked in the inventory.
    pub stackable: bool,
    /// Whether this item is a tool.
    pub tool: bool,
    /// Whether this item can be eaten.
    #[serde(default)]
    pub is_food: bool,
    /// The amount of hunger this item restores when eaten.
    #[serde(default)]
    pub food_value: f32,
    /// A map of other properties, such as damage for tools or health for structures.
    #[serde(default)]
    pub properties: Option<HashMap<String, f64>>,
    /// The category of the item, used for tasks like checking for a required tool type.
    #[serde(default)]
    pub category: Option<String>,
    /// The tier of the item, used for comparing tools. Higher is better.
    #[serde(default)]
    pub tier: u32,
}

pub struct ItemRegistry {
    pub items: HashMap<String, Item>,
}

impl ItemRegistry {
    pub fn new(filepath: &str) -> Result<Self, SimulationError> {
        let file_content = fs::read_to_string(filepath)?;
        let items_vec: Vec<Item> = serde_json::from_str(&file_content)?;
        let mut items = HashMap::new();
        for item in items_vec {
            items.insert(item.name.clone(), item);
        }
        Ok(ItemRegistry { items })
    }

    pub fn get_item(&self, item_name: &str) -> Option<&Item> {
        self.items.get(item_name)
    }
}
