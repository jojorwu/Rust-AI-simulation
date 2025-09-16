use bevy::prelude::*;
use rust_simulation::{
    components::{Position, Velocity},
    map::{Map, Tile},
    systems::movement::movement_system,
};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let map = Map::new(20, 20, "data/biomes.json", "data/resources.json")
        .expect("Failed to create map");
    app.insert_resource(map);
    app.add_systems(Update, movement_system);
    app
}

#[test]
fn test_movement_into_wall_is_blocked() {
    // 1. Setup
    let mut app = setup_test_app();

    // Create an agent next to the wall
    let agent_start_pos = Position { x: 5, y: 5 };
    let agent_entity = app
        .world
        .spawn((
            agent_start_pos,
            // Give it a velocity that would move it into the wall
            Velocity { dx: 0, dy: 1 },
        ))
        .id();

    // Get the map resource and modify it *after* spawning the entity
    let map = app.world.resource_mut::<Map>();

    // Place a non-walkable tile (a wall, which is not in the walkable set of is_walkable)
    let wall_pos = Position { x: 5, y: 6 };
    // Note: The 'walkable' boolean in Tile::new is not used by the old is_walkable method.
    // The tile_type '#' is what makes it non-walkable.
    map.set_tile(wall_pos.x, wall_pos.y, Tile::new('#', "rock".to_string()));
    map.add_entity_to_spatial_map(agent_entity, agent_start_pos.x, agent_start_pos.y);

    // 2. Run system
    app.update();

    // 3. Verify
    let agent_pos_after = app.world.get::<Position>(agent_entity).unwrap();

    // The agent should not have moved.
    assert_eq!(
        *agent_pos_after, agent_start_pos,
        "Agent should not move into a non-walkable tile"
    );

    // The velocity component should be removed, as the move was "resolved" (by failing).
    assert!(
        app.world.get::<Velocity>(agent_entity).is_none(),
        "Velocity component should be removed after a failed move attempt"
    );
}
