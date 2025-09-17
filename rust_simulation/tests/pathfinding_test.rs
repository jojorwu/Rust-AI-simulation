use bevy::prelude::*;
use rust_simulation::{
    brain::{Goal, MemoryTile},
    components::{
        ai::{ExplorationFrontier, MentalMap},
        path::{CurrentPath, PathRequest, PathfindingFailed},
        Position, intents::IntendsToExplore, BrainComponent
    },
    map::Tile,
    player::Player,
    systems::{
        pathfinding_completion_system::pathfinding_completion_system,
        pathfinding_system::pathfinding_system,
        ai::actions::explore::explore_action_system,
        pathfinding_failure::pathfinding_failure_system
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
fn test_long_pathfinding() {
    // Create a new Bevy App
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);

    let map = rust_simulation::map::Map::new(TEST_WIDTH, TEST_HEIGHT, "data/biomes.json", "data/resources.json").unwrap();
    app.insert_resource(map);

    // --- Setup Test Data ---
    // An empty mental map represents a wide open space
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
    let goal_pos = (50, 50); // A significantly long path
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
            mental_map,
        ))
        .id();

    // --- Run App ---
    // The pathfinding is async, so we need to run the app a few times.
    // This should be more than enough to get the result.
    for _ in 0..20 {
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // --- Check for Path ---
    let player_entity_ref = app.world.entity(player_entity);
    let current_path = player_entity_ref.get::<CurrentPath>();
    assert!(
        current_path.is_some(),
        "Path was not found for the long path test"
    );

    // Optional: Check if the path is correct
    let path = &current_path.unwrap().nodes;
    assert_eq!(path[0], start_pos, "Path does not start at the correct location");
    assert_eq!(path.back().unwrap(), &goal_pos, "Path does not end at the correct location");
    // Manhattan distance for a clear path
    assert_eq!(path.len(), 101, "Path length is not correct for a 50x50 journey");
}

#[test]
fn test_pathfinding_failure_clears_goal() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(rust_simulation::map::Map::new(TEST_WIDTH, TEST_HEIGHT, "data/biomes.json", "data/resources.json").unwrap());

    let mut map_data = std::collections::HashMap::new();
    // Wall off the goal
    map_data.insert((0, 1), MemoryTile { tile: Tile::new('#', "wall".to_string()) });
    map_data.insert((1, 1), MemoryTile { tile: Tile::new('#', "wall".to_string()) });
    map_data.insert((2, 1), MemoryTile { tile: Tile::new('#', "wall".to_string()) });
    map_data.insert((2, 0), MemoryTile { tile: Tile::new('#', "wall".to_string()) });

    let mental_map = MentalMap(Arc::new(map_data));

    app.add_systems(Update, (pathfinding_system, pathfinding_completion_system, pathfinding_failure_system).chain());

    let start_pos = (1, 0);
    let goal_pos = (1, 2);

    let mut brain = BrainComponent::new(
        Arc::new(
            RecipeManager::new("data/recipes.json")
                .expect("Failed to create recipe manager"),
        ),
        0.1,
        0.9,
        1.0,
    );
    brain.current_goal = Some(Goal::Explore);

    let agent_entity = app.world.spawn((
        Position { x: start_pos.0, y: start_pos.1 },
        PathRequest { start: start_pos, goal: goal_pos },
        mental_map,
        brain,
    )).id();

    // 2. Run the systems
    for _ in 0..10 {
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // 3. Verify
    let agent_brain = app.world.get::<BrainComponent>(agent_entity).unwrap();
    assert_eq!(agent_brain.current_goal, None, "Agent's goal should be cleared after pathfinding failure");

    let agent = app.world.entity(agent_entity);
    assert!(agent.get::<PathRequest>().is_none(), "PathRequest should be removed");
    assert!(agent.get::<PathfindingFailed>().is_none(), "PathfindingFailed should be removed");
}

#[test]
fn test_pathfinding_request_without_mental_map_fails_gracefully() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(rust_simulation::map::Map::new(TEST_WIDTH, TEST_HEIGHT, "data/biomes.json", "data/resources.json").unwrap());
    app.add_systems(Update, pathfinding_system);

    let start_pos = (0, 0);
    let goal_pos = (10, 10);

    // Entity has a PathRequest but no MentalMap
    let agent_entity = app.world.spawn((
        Position { x: start_pos.0, y: start_pos.1 },
        PathRequest { start: start_pos, goal: goal_pos },
    )).id();

    // 2. Run systems
    app.update();
    app.update(); // Apply commands

    // 3. Verify
    let agent = app.world.entity(agent_entity);
    // The system should have added a PathfindingFailed component.
    assert!(agent.get::<PathfindingFailed>().is_some(), "Pathfinding should fail for entity without a MentalMap");
    // The PathRequest should be removed.
    assert!(agent.get::<PathRequest>().is_none(), "PathRequest should be removed after failure");
}
