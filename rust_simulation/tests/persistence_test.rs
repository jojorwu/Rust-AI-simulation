use bevy::{app::AppExit, prelude::*};
use rust_simulation::{
    brain::{DiscretizedLevel, Goal, HighLevelState, InventorySummary},
    components::ai::GoalQTable,
    AppPaths,
    player::Player,
    systems::persistence::save_q_tables_on_exit,
};
use std::{
    collections::{BTreeMap, HashMap},
    panic,
};
use tempfile::tempdir;

#[test]
fn test_q_table_persistence_uses_app_paths() {
    // 1. Setup
    // Create a temporary directory for app data
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().to_path_buf();

    let expected_path = data_dir.join("q_tables.json");

    // Run the test in a closure to ensure cleanup happens even on panic.
    let result = panic::catch_unwind(|| {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()));
        app.add_event::<AppExit>();
        app.insert_resource(AppPaths {
            data_dir: data_dir.clone(),
        });
        app.add_systems(Update, save_q_tables_on_exit.run_if(on_event::<AppExit>()));

        // Create a dummy player with a Q-table to save
        let mut q_table = GoalQTable(HashMap::new());
        // Create a mock HighLevelState
        let state = HighLevelState {
            inventory_summary: InventorySummary {
                items: BTreeMap::new(),
            },
            num_hostile_players: 0,
            health_level: DiscretizedLevel::High,
            hunger_level: DiscretizedLevel::High,
            is_night: false,
        };
        // Create a mock Goal
        let goal = Goal::Explore;
        // Add some dummy data to ensure the file is not empty
        q_table.0.entry(state).or_default().insert(goal, 1.0);
        app.world
            .spawn((Player { id: 0, held_item: None }, q_table));


        // 2. Run the system by sending an AppExit event
        app.world.send_event(AppExit);
        app.update();
    });

    // 3. Verify
    assert!(result.is_ok(), "The system should not panic");
    assert!(
        expected_path.exists(),
        "q_tables.json should be saved in the directory specified by AppPaths, but it was not found there."
    );

    // 4. Cleanup is handled by temp_dir going out of scope
}
