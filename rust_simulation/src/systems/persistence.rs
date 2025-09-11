use crate::{components::ai::GoalQTable, player::Player};
use bevy_ecs::prelude::*;
use log::{error, info};
use std::{collections::HashMap, fs, io::Write};

// A helper function to encapsulate the actual saving logic and error handling.
fn save_q_tables(query: &Query<(&Player, &GoalQTable)>) -> Result<(), anyhow::Error> {
    info!("Inside save_q_tables. Query count: {}", query.iter().count());
    let mut q_tables_map = HashMap::new();
    for (player, q_table) in query.iter() {
        q_tables_map.insert(player.id, q_table.clone());
    }

    // Don't try to save if there's nothing to save.
    if q_tables_map.is_empty() {
        info!("Q-tables map is empty. Not saving.");
        return Ok(());
    }

    let final_path = "q_tables.json";
    let temp_path = "q_tables.json.tmp";

    // 1. Serialize data
    let json_data = serde_json::to_string_pretty(&q_tables_map)?;

    // 2. Write to a temporary file
    let mut temp_file = fs::File::create(temp_path)?;
    temp_file.write_all(json_data.as_bytes())?;

    // 3. Atomically rename the temporary file to the final destination
    fs::rename(temp_path, final_path)?;

    Ok(())
}

pub fn save_q_tables_on_exit(query: Query<(&Player, &GoalQTable)>) {
    info!("Saving Q-tables...");
    if let Err(e) = save_q_tables(&query) {
        error!("Failed to save Q-tables: {e}");
    } else {
        info!("Q-tables saved successfully to q_tables.json");
    }
}
