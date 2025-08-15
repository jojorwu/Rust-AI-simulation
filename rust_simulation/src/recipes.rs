use std::collections::HashMap;

pub fn get_recipes() -> HashMap<String, HashMap<String, u32>> {
    let mut recipes: HashMap<String, HashMap<String, u32>> = HashMap::new();

    // Stone Axe
    let mut r = HashMap::new();
    r.insert("wood".to_string(), 2);
    r.insert("stone".to_string(), 3);
    recipes.insert("stone_axe".to_string(), r);

    // Stone Pickaxe
    let mut r = HashMap::new();
    r.insert("wood".to_string(), 2);
    r.insert("stone".to_string(), 3);
    recipes.insert("stone_pickaxe".to_string(), r);

    // Furnace
    let mut r = HashMap::new();
    r.insert("stone".to_string(), 50);
    recipes.insert("furnace".to_string(), r);

    // Metal Pickaxe
    let mut r = HashMap::new();
    r.insert("iron_bars".to_string(), 10);
    r.insert("wood".to_string(), 2);
    recipes.insert("metal_pickaxe".to_string(), r);

    recipes
}
