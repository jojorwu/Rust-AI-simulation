mod map;
mod player;

use map::Map;
use player::Player;

fn main() {
    println!("--- Rust Simulation Test ---");

    // --- Map Generation Test ---
    const WIDTH: u32 = 60;
    const HEIGHT: u32 = 30;
    const SCALE: f64 = 25.0;
    const OCTAVES: i32 = 5;
    const PERSISTENCE: f64 = 0.5;
    const LACUNARITY: f64 = 2.0;

    println!("\n--- Generating Map... ---");
    let mut island_map = Map::new(WIDTH, HEIGHT);
    island_map.generate_island_map(SCALE, OCTAVES, PERSISTENCE, LACUNARITY);
    island_map.display();

    // --- Player and Inventory Test ---
    println!("\n--- Testing Player Inventory... ---");
    let mut test_player = Player::new(5, 5);

    println!("Initial Inventory: {:?}", test_player.inventory);

    // Add items
    test_player.add_item("wood", 10);
    test_player.add_item("stone", 5);
    test_player.add_item("wood", 5); // Add to existing stack
    test_player.add_item("stone_axe", 1); // Add a tool

    println!("Inventory after adding items: {:?}", test_player.inventory);

    // Check quantities
    let wood_count = test_player.get_total_quantity("wood");
    let stone_count = test_player.get_total_quantity("stone");
    let axe_count = test_player.get_total_quantity("stone_axe");

    println!("\nItem Quantities:");
    println!("Wood: {}", wood_count);
    println!("Stone: {}", stone_count);
    println!("Stone Axe: {}", axe_count);

    // Test removing resources
    let mut recipe = std::collections::HashMap::new();
    recipe.insert("wood".to_string(), 7);
    recipe.insert("stone".to_string(), 3);

    println!("\nAttempting to remove resources for recipe: {:?}", recipe);
    if test_player.has_resources(&recipe) {
        if test_player.remove_resources(&recipe) {
            println!("Successfully removed resources.");
        } else {
            println!("Failed to remove resources.");
        }
    } else {
        println!("Not enough resources for recipe.");
    }

    println!("Final Inventory: {:?}", test_player.inventory);
    let final_wood_count = test_player.get_total_quantity("wood");
    println!("Final wood count: {}", final_wood_count);
}
