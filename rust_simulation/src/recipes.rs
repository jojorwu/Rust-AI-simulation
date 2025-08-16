use std::collections::HashMap;
use std::fs;

pub struct RecipeManager {
    pub recipes: HashMap<String, HashMap<String, u32>>,
}

impl RecipeManager {
    pub fn new(filepath: &str) -> Self {
        let file_content = fs::read_to_string(filepath)
            .expect(&format!("Unable to read recipes file at {}", filepath));
        let recipes: HashMap<String, HashMap<String, u32>> = serde_json::from_str(&file_content)
            .expect("Unable to parse recipes.json");

        RecipeManager { recipes }
    }

    pub fn get_required_resources(&self, item: &str, quantity: u32) -> HashMap<String, u32> {
        let mut required_resources = HashMap::new();

        if let Some(recipe) = self.recipes.get(item) {
            for _ in 0..quantity {
                for (ingredient, &ing_quantity) in recipe {
                    let sub_resources = self.get_required_resources(ingredient, ing_quantity);
                    for (sub_resource, sub_quantity) in sub_resources {
                        *required_resources.entry(sub_resource).or_insert(0) += sub_quantity;
                    }
                }
            }
        } else {
            required_resources.insert(item.to_string(), quantity);
        }

        required_resources
    }
}
