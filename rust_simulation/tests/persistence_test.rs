use bevy::{app::AppExit, prelude::*};
use rust_simulation::{
    brain::{Goal, HighLevelState, InventorySummary},
    components::ai::GoalQTable,
    player::Player,
    systems::persistence::save_q_tables_on_exit,
};
use std::{collections::HashMap, fs, panic};

#[test]
fn test_q_table_persistence() {
    let test_file = "q_tables.json";

    // Run the test in a closure to ensure cleanup happens even on panic.
    let result = panic::catch_unwind(|| {
        // 1. Setup
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()));
        app.add_event::<AppExit>();
        app.add_systems(Update, save_q_tables_on_exit.run_if(on_event::<AppExit>()));

        // Create a mock HighLevelState
        let state = HighLevelState {
            inventory_summary: InventorySummary {
                wood: 1,
                stone: 2,
                iron_ore: 3,
                stone_axe: 4,
            },
            num_hostile_players: 0,
            health_level: 100,
            hunger_level: 0,
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
                _held_item: None,
            },
            q_table,
        ));

        // 2. Run the system by sending an AppExit event
        app.world.send_event(AppExit);
        app.update();

        // 3. Verify the output file
        let content = fs::read_to_string(test_file).expect("Failed to read test file");
        let deserialized: HashMap<u32, GoalQTable> =
            serde_json::from_str(&content).expect("Failed to deserialize test data");

        assert_eq!(deserialized.len(), 1);
        assert!(deserialized.contains_key(&1));
        let saved_q_table = deserialized.get(&1).unwrap();
        let saved_goal_map = saved_q_table.0.get(&state).unwrap();
        assert_eq!(saved_goal_map.get(&goal), Some(&42.0));
    });

    // 4. Cleanup
    if fs::metadata(test_file).is_ok() {
        fs::remove_file(test_file).unwrap();
    }
    let temp_file = "q_tables.json.tmp";
    if fs::metadata(temp_file).is_ok() {
        fs::remove_file(temp_file).unwrap();
    }

    assert!(result.is_ok());
}
