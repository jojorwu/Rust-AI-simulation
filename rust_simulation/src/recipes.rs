use crate::errors::SimulationError;
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct RecipeManager {
    pub recipes: HashMap<String, HashMap<String, u32>>,
}

impl RecipeManager {
    pub fn new(filepath: &str) -> Result<Self, SimulationError> {
        let file_content = fs::read_to_string(filepath)?;
        let recipes: HashMap<String, HashMap<String, u32>> = serde_json::from_str(&file_content)?;

        Ok(RecipeManager { recipes })
    }

    pub fn with_recipes(recipes: HashMap<String, HashMap<String, u32>>) -> Self {
        RecipeManager { recipes }
    }

    // Public wrapper function
    pub fn get_required_resources(
        &self,
        item: &str,
        quantity: u32,
    ) -> Result<HashMap<String, u32>, SimulationError> {
        self.get_required_resources_recursive(item, quantity, &mut HashSet::new())
    }

    // Private recursive function with cycle detection
    fn get_required_resources_recursive(
        &self,
        item: &str,
        quantity: u32,
        visited: &mut HashSet<String>,
    ) -> Result<HashMap<String, u32>, SimulationError> {
        if visited.contains(item) {
            return Err(SimulationError::CircularDependency(item.to_string()));
        }
        visited.insert(item.to_string());

        let mut required_resources = HashMap::new();

        if let Some(recipe) = self.recipes.get(item) {
            for _ in 0..quantity {
                for (ingredient, &ing_quantity) in recipe {
                    let sub_resources = self.get_required_resources_recursive(
                        ingredient,
                        ing_quantity,
                        visited,
                    )?;
                    for (sub_resource, sub_quantity) in sub_resources {
                        *required_resources.entry(sub_resource).or_insert(0) += sub_quantity;
                    }
                }
            }
        } else {
            required_resources.insert(item.to_string(), quantity);
        }

        visited.remove(item);
        Ok(required_resources)
    }
}
