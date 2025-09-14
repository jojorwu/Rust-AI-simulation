use bevy::prelude::*;
use rust_simulation::{
    brain::{Goal, HighLevelState, InventorySummary},
    components::{ai::GoalQTable, BrainComponent},
    config::Config,
    events::Event,
    systems::ai::q_learning::update_q_table_system,
};
use std::collections::{BTreeMap, HashMap};

#[test]
fn test_q_table_update_ignores_invalid_future_goals() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();

    // Add resources
    let config = Config::load("data/config.toml").expect("Failed to load config");
    app.insert_resource(config);
    let map =
        rust_simulation::map::Map::new(1, 1, "data/biomes.json", "data/resources.json").unwrap();
    app.insert_resource(map);

    // Create an agent with a brain and a Q-table
    let recipe_manager =
        std::sync::Arc::new(rust_simulation::recipes::RecipeManager::new("data/recipes.json").unwrap());
    let brain = BrainComponent::new(recipe_manager, 0.1, 0.9, 0.1);
    let mut q_table = GoalQTable(HashMap::new());

    // Define two states
    let prev_state = HighLevelState {
        inventory_summary: InventorySummary {
            items: BTreeMap::new(),
        },
        num_hostile_players: 0,
        health_level: rust_simulation::brain::DiscretizedLevel::High,
        hunger_level: rust_simulation::brain::DiscretizedLevel::High,
        is_night: false,
    };
    let new_state = HighLevelState {
        inventory_summary: InventorySummary {
            items: BTreeMap::new(),
        },
        num_hostile_players: 0,
        health_level: rust_simulation::brain::DiscretizedLevel::Medium,
        hunger_level: rust_simulation::brain::DiscretizedLevel::High,
        is_night: false,
    };

    // Pre-populate the Q-table for the *new* state.
    // The best goal is to gather wood, but we'll make this invalid later.
    let mut future_q_values = HashMap::new();
    future_q_values.insert(Goal::GatherResource("wood".to_string(), 1), 100.0); // High value, but invalid
    future_q_values.insert(Goal::Explore, 10.0); // Low value, but valid
    q_table.0.insert(new_state.clone(), future_q_values);

    let agent_entity = app
        .world
        .spawn((
            brain,
            q_table,
            rust_simulation::components::ai::KnownResources(HashMap::new()),
        ))
        .id();

    // Create the event that will trigger the update
    let mut events = app.world.resource_mut::<Events<Event>>();
    events.send(Event::GoalCompleted {
        entity: agent_entity,
        prev_state: prev_state.clone(),
        goal: Goal::Explore, // The goal that led from prev_state to new_state
        new_state: new_state.clone(),
        reward: 5.0,
    });

    // Add the system to run
    app.add_systems(Update, update_q_table_system);

    // 2. Run the system
    app.update();

    // 3. Verify
    // The system should have calculated the max_future_q based on the *valid* goals.
    // The valid goal is Explore, with a Q-value of 10.0.
    // The invalid goal GatherResource should be ignored.
    // Formula: new_q = old_q + alpha * (reward + gamma * max_future_q - old_q)
    // new_q = 0.0 + 0.1 * (5.0 + 0.9 * 10.0 - 0.0)
    // new_q = 0.1 * (5.0 + 9.0)
    // new_q = 0.1 * 14.0 = 1.4
    let expected_q_value = 1.4;

    let final_q_table = app.world.get::<GoalQTable>(agent_entity).unwrap();
    let updated_q_value = final_q_table
        .0
        .get(&prev_state)
        .unwrap()
        .get(&Goal::Explore)
        .unwrap();

    assert!((updated_q_value - expected_q_value).abs() < 1e-6);
}
