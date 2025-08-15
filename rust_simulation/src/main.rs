mod map;
use map::Map;

fn main() {
    const WIDTH: u32 = 60; // Use a larger map for better visualization
    const HEIGHT: u32 = 30;

    // Noise parameters
    const SCALE: f64 = 25.0;
    const OCTAVES: i32 = 5;
    const PERSISTENCE: f64 = 0.5;
    const LACUNARITY: f64 = 2.0;

    println!("--- Generating Rust Island Map ---");

    let mut island_map = Map::new(WIDTH, HEIGHT);
    island_map.generate_island_map(SCALE, OCTAVES, PERSISTENCE, LACUNARITY);

    println!("\n--- Map Display ---");
    island_map.display();
}
