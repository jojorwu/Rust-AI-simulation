use rust_simulation::{
    brain::Goal,
    components::{ai::KnownResources, BrainComponent, Inventory},
    recipes::RecipeManager,
    systems::ai::goal_selection::plan_goal,
};
use std::{collections::HashMap, sync::Arc};

#[test]
fn test_plan_goal_craft_item() {
    let recipe_manager = Arc::new(RecipeManager::new("data/recipes.json").unwrap());
    let brain = BrainComponent::new(Arc::clone(&recipe_manager), 0.1, 0.9, 1.0);
    let inventory = Inventory::new();
    let known_resources = KnownResources(HashMap::new());
    let goal = Goal::CraftItem("stone_axe".to_string());

    let plan = plan_goal(&brain, &inventory, &known_resources, &goal).unwrap();

    assert_eq!(plan.len(), 5);
    assert!(plan.contains(&Goal::Explore));
    assert!(plan.contains(&Goal::GatherResource("wood".to_string())));
    assert!(plan.contains(&Goal::GatherResource("stone".to_string())));
    assert_eq!(plan[4], Goal::CraftItem("stone_axe".to_string()));
}
