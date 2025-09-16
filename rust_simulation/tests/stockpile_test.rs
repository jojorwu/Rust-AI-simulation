use bevy::prelude::*;
use rust_simulation::{
    components::{
        intents::IntendsToStockpile,
        path::PathRequest,
        BrainComponent, Chest, Position,
    },
    systems::ai::actions::stockpile::stockpile_action_system,
    RecipeManagerResource,
};
use std::sync::Arc;

#[test]
fn test_stockpile_chooses_closest_chest_to_agent() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, stockpile_action_system);

    // Dummy recipe manager for BrainComponent
    let recipe_manager = Arc::new(
        rust_simulation::recipes::RecipeManager::new("data/recipes.json").unwrap(),
    );
    app.insert_resource(RecipeManagerResource(recipe_manager.clone()));

    // Agent is at (1,1), but its home base is at (100,100)
    let agent_pos = Position { x: 1, y: 1 };
    let home_base_pos = Position { x: 100, y: 100 };

    // Chest A is far away, near the home base
    let far_chest_pos = Position { x: 101, y: 101 };
    app.world.spawn((Chest { inventory: Default::default() }, far_chest_pos));

    // Chest B is close, but not adjacent
    let near_chest_pos = Position { x: 5, y: 5 };
    app.world.spawn((Chest { inventory: Default::default() }, near_chest_pos));

    // Create the agent
    let mut brain = BrainComponent::new(recipe_manager, 0.1, 0.9, 0.1);
    brain.home_base = Some(home_base_pos);

    let agent_entity = app
        .world
        .spawn((
            brain,
            agent_pos,
            IntendsToStockpile("wood".to_string()),
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let agent = app.world.entity(agent_entity);
    let path_request = agent.get::<PathRequest>().expect("Agent should have a PathRequest");

    // The goal of the path request should be the nearby chest, not the far one.
    assert_eq!(
        path_request.goal,
        (near_chest_pos.x, near_chest_pos.y),
        "Agent should path to the chest closest to its current position, not its home base"
    );
}
