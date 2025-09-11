use rust_simulation::map::Map;

#[test]
fn test_map_generation_is_deterministic() {
    let map1 = Map::new(10, 10, "data/biomes.json", "data/resources.json", Some(123)).unwrap();
    let map2 = Map::new(10, 10, "data/biomes.json", "data/resources.json", Some(123)).unwrap();

    // With the same seed, the maps should be identical
    assert_eq!(map1.get_tile(5, 5), map2.get_tile(5, 5));
}

#[test]
fn test_map_generation_with_seed() {
    // This test will fail until the seed is used
    let map1 = Map::new(10, 10, "data/biomes.json", "data/resources.json", Some(123)).unwrap();
    let map2 = Map::new(10, 10, "data/biomes.json", "data/resources.json", Some(456)).unwrap();

    // With a different seed, the maps should be different
    assert_ne!(map1.get_tile(5, 5), map2.get_tile(5, 5));
}
