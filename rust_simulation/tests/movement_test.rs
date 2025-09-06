use bevy::prelude::*;
use rust_simulation::{
    components::{Position, Velocity},
    map::{Map, Tile},
    systems::movement::movement_system,
};

#[test]
fn test_movement_into_wall() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let map = Map::new(10, 10, "data/biomes.json", "data/resources.json")
        .expect("Failed to create map");
    // Create a wall at (5, 5)
    map.set_tile(5, 5, Tile::new('#', "wall".to_string()));
    app.insert_resource(map);

    app.add_systems(Update, movement_system);

    let start_pos = Position { x: 4, y: 5 };
    let entity = app
        .world
        .spawn((start_pos, Velocity { dx: 1, dy: 0 }))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let final_pos = app.world.get::<Position>(entity).unwrap();
    // The entity should not have moved because the destination is a wall.
    assert_eq!(*final_pos, start_pos);
}
