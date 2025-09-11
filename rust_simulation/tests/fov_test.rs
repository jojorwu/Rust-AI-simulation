use rust_simulation::{
    components::Position,
    fov::field_of_view,
    map::{Map, Tile},
};

#[test]
fn test_fov_wall_occlusion() {
    let map = Map::new(10, 10, "data/biomes.json", "data/resources.json").unwrap();

    // Create a horizontal wall
    for x in 0..5 {
        map.set_tile(x, 5, Tile::new('#', "wall".to_string()));
    }

    let player_pos = Position { x: 2, y: 2 };
    let visible_tiles = field_of_view(&player_pos, 5, &map);

    // A tile behind the wall should not be visible
    let behind_wall_pos = Position { x: 2, y: 6 };
    assert!(!visible_tiles.contains(&behind_wall_pos));

    // A tile in front of the wall should be visible
    let in_front_of_wall_pos = Position { x: 2, y: 4 };
    assert!(visible_tiles.contains(&in_front_of_wall_pos));
}
