use bevy::{app::AppExit, prelude::*};
use rust_simulation::{
    brain::{DiscretizedLevel, Goal, HighLevelState, InventorySummary},
    components::ai::GoalQTable,
    player::Player,
};
use std::{collections::{BTreeMap, HashMap}, fs, panic};

// A helper struct to ensure that files are cleaned up, even on panic.
struct TestFile<'a> {
    path: &'a str,
}

impl<'a> Drop for TestFile<'a> {
    fn drop(&mut self) {
        if fs::metadata(self.path).is_ok() {
            if fs::metadata(self.path).unwrap().is_dir() {
                fs::remove_dir_all(self.path).expect("Failed to remove test directory");
            } else {
                fs::remove_file(self.path).expect("Failed to remove test file");
            }
        }
        let temp_path = format!("{}.tmp", self.path);
        if fs::metadata(&temp_path).is_ok() {
            fs::remove_file(&temp_path).expect("Failed to remove temp file");
        }
    }
}

#[test]
fn test_q_table_persistence() {
    let test_file = "persistence_test_1.json";
    let _guard = TestFile { path: test_file };

    // Run the test in a closure to ensure cleanup happens even on panic.
    let result = panic::catch_unwind(|| {
        // 1. Setup
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()));
        app.add_event::<AppExit>();
        app.add_systems(PostUpdate, move |query: Query<(&Player, &GoalQTable)>| {
            if let Err(e) = rust_simulation::systems::persistence::save_q_tables(&query, test_file) {
                panic!("Failed to save Q-tables: {e}");
            }
        });

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
        let content = fs::read_to_string(test_file).expect("Failed to read test file");
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

#[test]
fn test_q_table_persistence_cleanup_on_rename_error() {
    let test_file = "persistence_test_2.json";
    let _guard = TestFile { path: test_file };

    // Create a directory where the final file should be, to cause a rename error.
    fs::create_dir_all(test_file).expect("Failed to create test directory");

    let result = panic::catch_unwind(|| {
        // 1. Setup
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()));
        app.add_event::<AppExit>();
        app.add_systems(PostUpdate, move |query: Query<(&Player, &GoalQTable)>| {
            // We expect this to fail, so we don't panic.
            let _ = rust_simulation::systems::persistence::save_q_tables(&query, test_file);
        });

        // Create a mock Q-table
        let mut q_table = GoalQTable(HashMap::new());
        q_table.0.insert(
            HighLevelState {
                inventory_summary: InventorySummary { items: BTreeMap::new() },
                num_hostile_players: 0,
                health_level: DiscretizedLevel::High,
                hunger_level: DiscretizedLevel::Low,
                is_night: false,
            },
            HashMap::new(),
        );

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

        // 3. Verify that the temp file does not exist
        let temp_file = format!("{}.tmp", test_file);
        assert!(!fs::metadata(&temp_file).is_ok(), "Temp file should be cleaned up on rename error");
    });

    assert!(result.is_ok());
}
