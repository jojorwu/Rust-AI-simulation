use crate::{components::ai::GoalQTable, player::Player};
use bevy_ecs::prelude::*;
use log::info;
use std::{collections::HashMap, fs::File, io::Write};

pub fn save_q_tables_on_exit(query: Query<(&Player, &GoalQTable)>) {
    let mut q_tables_map = HashMap::new();
    for (player, q_table) in query.iter() {
        q_tables_map.insert(player.id, q_table.clone());
    }

    let json_data = serde_json::to_string_pretty(&q_tables_map).unwrap();
    let mut file = File::create("q_tables.json").unwrap();
    file.write_all(json_data.as_bytes()).unwrap();
    info!("Q-tables saved to q_tables.json");
}
