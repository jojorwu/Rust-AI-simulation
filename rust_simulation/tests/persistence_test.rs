use bevy::{app::AppExit, prelude::*};
use rust_simulation::{
    brain::{DiscretizedLevel, Goal, HighLevelState, InventorySummary},
    components::ai::GoalQTable,
    player::Player,
    systems::persistence::save_q_tables_on_exit,
    AppPaths,
};
use std::{collections::{BTreeMap, HashMap}, fs, panic};
use tempfile::tempdir;

#[test]
fn test_q_table_persistence() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().to_path_buf();
    let test_file = data_dir.join("q_tables.json");

    // Run the test in a closure to ensure cleanup happens even on panic.
    let result = panic::catch_unwind(|| {
        // 1. Setup
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()));
        app.add_event::<AppExit>();
        app.insert_resource(AppPaths {
            data_dir: data_dir.clone(),
        });
        app.add_systems(Update, save_q_tables_on_exit.run_if(on_event::<AppExit>()));

        // Create a mock HighLevelState
        let mut items = BTreeMap::new();
        items.insert("wood".to_string(), 1);
        items.insert("stone".to_string(), 2);
        items.insert("iron_ore".to_string(), 3);
        items.insert("stone_axe".to_string(), 4);
        let state = HighLevelState {
            inventory_summary: InventorySummary { items },
            num_hostile_players: 0,
            health_level: DiscretizedLevel::High,
            hunger_level: DiscretizedLevel::Low,
            is_night: false,
        };

        // Create a mock Goal
        let goal = Goal::Explore;

        // Create a mock inner HashMap
        let mut goal_map = HashMap::new();
        goal_map.insert(goal.clone(), 42.0);

        // Create a mock Q-table
        let mut q_table = GoalQTable(HashMap::new());
        q_table.0.insert(state.clone(), goal_map);

        // Create a mock player entity
        app.world.spawn((
            Player {
                id: 1,
                held_item: None,
            },
            q_table,
        ));

        // 2. Run the system by sending an AppExit event
        app.world.send_event(AppExit);
        app.update();

        // 3. Verify the output file
        let content = fs::read_to_string(&test_file).expect("Failed to read test file");
        let deserialized: HashMap<u32, GoalQTable> =
            serde_json::from_str(&content).expect("Failed to deserialize test data");

        assert_eq!(deserialized.len(), 1);
        assert!(deserialized.contains_key(&1));
        let saved_q_table = deserialized
            .get(&1)
            .expect("Player 1's Q-table should be present");
        let saved_goal_map = saved_q_table
            .0
            .get(&state)
            .expect("State should be present in the Q-table");
        assert_eq!(saved_goal_map.get(&goal), Some(&42.0));
    });

    assert!(result.is_ok());
}
