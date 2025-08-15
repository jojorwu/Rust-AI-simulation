mod map;
mod player;
mod state;
mod agent;

use map::Map;
use player::{Player, Slot};
use state::StateKey;
use agent::Agent;

fn main() {
    println!("--- Rust Simulation Test ---");

    // --- Map Generation Test ---
    println!("\n--- Generating Map... ---");
    let mut island_map = Map::new(20, 10);
    island_map.generate_island_map(10.0, 5, 0.5, 2.0);
    island_map.display();

    // --- Player and Inventory Test ---
    println!("\n--- Testing Player Inventory... ---");
    let mut test_player = Player::new(5, 5);
    test_player.add_item("wood", 10);
    test_player.add_item("stone_axe", 1);
    println!("Player Inventory: {:?}", test_player.inventory);

    // --- Agent and Q-Learning Test ---
    println!("\n--- Testing Agent Logic... ---");
    let actions = vec![
        "up".to_string(), "down".to_string(), "left".to_string(), "right".to_string(),
        "gather".to_string(), "craft_stone_axe".to_string()
    ];
    let mut agent = Agent::new(actions.clone(), 0.1, 0.9, 1.0);

    // Create a sample state
    let state1 = StateKey {
        local_view: vec!['.', 'T', '.'],
        inventory: test_player.inventory.clone(),
        held_item: None,
    };

    // Choose an action
    let action = agent.choose_action(&state1);
    println!("Agent chose action: {}", action);

    // Simulate a transition
    let reward = 20.0; // Got wood
    test_player.add_item("wood", 1);
    let state2 = StateKey {
        local_view: vec!['.', '.', '.'], // Tree is gone
        inventory: test_player.inventory.clone(),
        held_item: None,
    };

    println!("\nQ-table before update: {:?}", agent.q_table);
    agent.update_q_table(&state1, &action, reward, &state2);
    println!("Q-table after update: {:?}", agent.q_table);

    // Choose another action from the new state
    let action2 = agent.choose_action(&state2);
    println!("\nAgent chose new action from state 2: {}", action2);
}
