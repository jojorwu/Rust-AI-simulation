use bevy::prelude::*;
use rust_simulation::{
    brain::MemoryTile,
    components::{
        ai::{ExplorationFrontier, MentalMap},
        path::{CurrentPath, PathRequest},
        Position, BrainComponent, intents::IntendsToExplore
    },
    map::Tile,
    player::Player,
    systems::{
        pathfinding_completion_system::pathfinding_completion_system,
        pathfinding_system::pathfinding_system,
        ai::actions::explore::explore_action_system,
    },
    recipes::RecipeManager,
};
use std::{collections::VecDeque, sync::Arc};


const TEST_WIDTH: u32 = 100;
const TEST_HEIGHT: u32 = 100;

#[test]
fn test_pathfinding_flow() {
    // Create a new Bevy App
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);

    let map = rust_simulation::map::Map::new(TEST_WIDTH, TEST_HEIGHT, "data/biomes.json", "data/resources.json").unwrap();
    app.insert_resource(map);

    // --- Setup Test Data ---
    let mut map_data = std::collections::HashMap::new();
    // Create a wall
    map_data.insert(
        (1, 1),
        MemoryTile {
            tile: Tile::new('#', "wall".to_string()),
        },
    );
    let mental_map = MentalMap(Arc::new(map_data));

    // --- Setup Systems ---
    app.add_systems(
        Update,
        (
            pathfinding_system,
            pathfinding_completion_system,
        ),
    );

    // --- Setup Entities ---
    let start_pos = (0, 0);
    let goal_pos = (4, 0);
    let player_entity = app
        .world
        .spawn((
            Player::new(0, TEST_WIDTH, TEST_HEIGHT),
            Position {
                x: start_pos.0,
                y: start_pos.1,
            },
            PathRequest {
                start: start_pos,
                goal: goal_pos,
            },
            mental_map, // Add the mental map component
        ))
        .id();

    // --- Run App ---
    for _ in 0..10 {
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // --- Check for Path ---
    let player_entity_ref = app.world.entity(player_entity);
    assert!(
        player_entity_ref.get::<CurrentPath>().is_some(),
        "Path was not found"
    );
}

#[test]
fn test_exploration_flow() {
    // Create a new Bevy App
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);

    // --- Setup Test Data ---
    let mental_map = MentalMap(Arc::new(std::collections::HashMap::new()));
    let mut exploration_frontier = ExplorationFrontier(VecDeque::new());
    // Manually add a frontier for the test
    exploration_frontier.0.push_back(Position { x: 1, y: 0 });

    // --- Setup Systems ---
    app.add_systems(Update, explore_action_system);

    // --- Setup Entities ---
    let start_pos = (0, 0);
    let player_entity = app
        .world
        .spawn((
            Player::new(0, TEST_WIDTH, TEST_HEIGHT),
            Position {
                x: start_pos.0,
                y: start_pos.1,
            },
            IntendsToExplore,
            mental_map,
            exploration_frontier,
            // Add an empty brain component to prevent panics
            BrainComponent::new(
                Arc::new(
                    RecipeManager::new("data/recipes.json")
                        .expect("Failed to create recipe manager"),
                ),
                0.1,
                0.9,
                1.0,
            ),
        ))
        .id();

    // --- Run App ---
    app.update();

    // --- Check for PathRequest ---
    let player_entity_ref = app.world.entity(player_entity);
    assert!(
        player_entity_ref.get::<PathRequest>().is_some(),
        "PathRequest was not generated"
    );
}

#[test]
fn test_long_path_does_not_overflow_stack() {
    // Create a new Bevy App
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);

    let map = rust_simulation::map::Map::new(200, 200, "data/biomes.json", "data/resources.json").unwrap();
    app.insert_resource(map);

    // --- Setup Test Data ---
    let mental_map = MentalMap(Arc::new(std::collections::HashMap::new()));

    // --- Setup Systems ---
    app.add_systems(
        Update,
        (
            pathfinding_system,
            pathfinding_completion_system,
        ),
    );

    // --- Setup Entities ---
    let start_pos = (0, 0);
    let goal_pos = (199, 199);
    let player_entity = app
        .world
        .spawn((
            Player::new(0, 200, 200),
            Position {
                x: start_pos.0,
                y: start_pos.1,
            },
            PathRequest {
                start: start_pos,
                goal: goal_pos,
            },
            mental_map,
        ))
        .id();

    // --- Run App ---
    for _ in 0..20 { // More iterations for a longer path
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    // --- Check for Path ---
    let player_entity_ref = app.world.entity(player_entity);
    assert!(
        player_entity_ref.get::<CurrentPath>().is_some(),
        "Path was not found on a long path"
    );
}
