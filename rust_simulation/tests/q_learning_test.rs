use bevy::prelude::*;
use rust_simulation::{
    brain::Goal,
    components::{
        ai::{ExplorationFrontier, GoalQTable, PlayerMemories, KnownResources},
        status::{Health, Hunger},
        BrainComponent, Inventory, Position,
    },
    events::Event,
    systems::{
        ai::{
            goal_completion::goal_completion_system,
            goal_selection::goal_selection_system,
            q_learning::update_q_table_system,
        },
        gathering::gathering_system,
    },
    RecipeManagerResource,
};
use std::{collections::{HashMap, VecDeque}, sync::Arc};

#[test]
fn test_goal_completed_event_is_sent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();
    app.insert_resource(RecipeManagerResource(Arc::new(
        rust_simulation::recipes::RecipeManager::new("data/recipes.json").unwrap(),
    )));
    app.insert_resource(rust_simulation::config::Config::load("data/config.toml").unwrap());
    app.insert_resource(rust_simulation::ItemRegistryResource(Arc::new(
        rust_simulation::item::ItemRegistry::new("data/items.json").unwrap(),
    )));
    app.insert_resource(rust_simulation::map::Map::new(
        10,
        10,
        "data/biomes.json",
        "data/resources.json",
    ).unwrap());
    app.insert_resource(rust_simulation::IsDay(true));

    app.add_systems(
        Update,
        (
            goal_selection_system,
            gathering_system,
            goal_completion_system,
            update_q_table_system,
        )
            .chain(),
    );

    let entity = app
        .world
        .spawn((
            BrainComponent::new(
                app.world.resource::<RecipeManagerResource>().0.clone(),
                0.1,
                0.9,
                0.1,
            ),
            GoalQTable(HashMap::new()),
            Health {
                current: 100,
                max: 100,
            },
            Hunger {
                current: 100.0,
                max: 100.0,
            },
            Inventory::new(),
            Position { x: 0, y: 0 },
            KnownResources(HashMap::new()),
            PlayerMemories(HashMap::new()),
            ExplorationFrontier(VecDeque::new()),
        ))
        .id();

    // The AI will select a goal. Since there are no resources, it will likely choose to explore.
    // The goal stack for explore is empty, so it should complete immediately.
    // The agent will then select a new goal.
    // A GoalCompleted event should be sent for the failed goal.
    app.update();

    let events = app.world.resource::<Events<Event>>();
    let mut reader = events.get_reader();
    let mut event_found = false;
    for event in reader.read(events) {
        if let Event::GoalCompleted { .. } = event {
            event_found = true;
        }
    }
    assert!(event_found, "GoalCompleted event was not sent");
}
