use crate::errors::SimulationError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub name: String,
    pub stackable: bool,
    pub tool: bool,
    #[serde(default)]
    pub properties: Option<HashMap<String, f64>>,
}

use bevy_ecs::prelude::Resource;

#[derive(Resource)]
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
