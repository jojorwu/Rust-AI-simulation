use bevy::prelude::*;
use rust_simulation::{components::Position, fov, map::Map};
use std::collections::HashSet;

fn create_test_map(walls: &[(u32, u32)]) -> Map {
    let mut map = Map::new(10, 10, "data/biomes.json", "data/resources.json")
        .expect("Failed to create map");
    for &(x, y) in walls {
        map.set_tile(x, y, rust_simulation::map::Tile::new('#', "wall".to_string()));
    }
    map
}

#[test]
fn test_fov_sees_around_corners() {
    let walls = vec![(1, 1)];
    let map = create_test_map(&walls);
    let player_pos = Position { x: 0, y: 0 };
    let visible_tiles = fov::field_of_view(&player_pos, 5, &map);

    let expected_visible: HashSet<Position> = [
        (0, 0), (1, 0), (0, 1), (2, 1), (1, 2)
    ]
    .iter()
    .map(|&(x, y)| Position { x, y })
    .collect();

    assert!(
        visible_tiles.contains(&Position { x: 1, y: 2 }),
        "Should see the tile at (1, 2)"
    );
    assert!(
        visible_tiles.contains(&Position { x: 2, y: 1 }),
        "Should see the tile at (2, 1)"
    );
}
