use std::collections::HashMap;
use std::fs;

pub fn get_recipes() -> HashMap<String, HashMap<String, u32>> {
    let file_content = fs::read_to_string("recipes.json").expect("Unable to read recipes.json");
    serde_json::from_str(&file_content).expect("Unable to parse recipes.json")
}
