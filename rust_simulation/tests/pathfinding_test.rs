use bevy::prelude::*;
use rust_simulation::{
    async_task::AsyncResultChannel,
    brain::MemoryTile,
    components::{
        ai::{ExplorationFrontier, MentalMap},
        intents::IntendsToExplore,
        path::{CurrentPath, PathRequest},
        Position, BrainComponent,
    },
    map::Tile,
    player::Player,
    recipes::RecipeManager,
    systems::{
        ai::actions::explore::explore_action_system,
        async_result_collection_system::async_result_collection_system,
        path_collection_system::path_collection_system,
        pathfinding_system::pathfinding_system,
    },
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

    // --- Setup Test Data ---
    let mut mental_map = MentalMap(vec![vec![None; TEST_WIDTH as usize]; TEST_HEIGHT as usize]);
    // Create a walkable path
    for i in 0..5 {
        mental_map.0[0][i] = Some(MemoryTile {
            tile: Tile::new('.', "grassland".to_string()),
        });
    }
    // Create a wall
    mental_map.0[1][1] = Some(MemoryTile {
        tile: Tile::new('#', "wall".to_string()),
    });

    app.init_resource::<AsyncResultChannel>();

    // --- Setup Systems ---
    app.add_systems(
        Update,
        (
            pathfinding_system,
            async_result_collection_system,
            path_collection_system,
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
    let mental_map = MentalMap(vec![vec![None; TEST_WIDTH as usize]; TEST_HEIGHT as usize]);
    let mut exploration_frontier = ExplorationFrontier(VecDeque::new());
    // Manually add a frontier for the test
    exploration_frontier.0.push_back(Position { x: 1, y: 0 });

    app.init_resource::<AsyncResultChannel>();

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
                Arc::new(RecipeManager::new("data/recipes.json").unwrap()),
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
