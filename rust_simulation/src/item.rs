use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Item {
    pub name: String,
    pub stackable: bool,
    pub tool: bool,
}

pub struct ItemRegistry {
    pub items: HashMap<String, Item>,
}

impl ItemRegistry {
    pub fn new(filepath: &str) -> Self {
        let file_content = fs::read_to_string(filepath).expect("Unable to read items.json");
        let items_vec: Vec<Item> = serde_json::from_str(&file_content).expect("Unable to parse items.json");
        let mut items = HashMap::new();
        for item in items_vec {
            items.insert(item.name.clone(), item);
        }
        ItemRegistry { items }
    }

    pub fn get_item(&self, item_name: &str) -> Option<&Item> {
        self.items.get(item_name)
    }
}
