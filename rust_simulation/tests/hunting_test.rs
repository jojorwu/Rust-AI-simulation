use bevy::prelude::*;
use rust_simulation::{
    animals::pig::Pig,
    components::{
        intents::{IntendsToGather, WantsToAttack},
        path::PathRequest,
        Position,
    },
    systems::hunting::hunting_system,
};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(rust_simulation::map::Map::new(20, 20, "data/biomes.json", "data/resources.json").unwrap());
    app.add_systems(Update, hunting_system);
    app
}

#[test]
fn test_hunting_system_path_request() {
    // 1. Setup
    let mut app = setup_test_app();

    let pig_pos = Position { x: 5, y: 5 };
    let pig_entity = app.world.spawn((Pig, pig_pos)).id();
    app.world.resource_mut::<rust_simulation::map::Map>().add_entity_to_spatial_map(pig_entity, pig_pos.x, pig_pos.y);
    let hunter_entity = app
        .world
        .spawn((
            Position { x: 0, y: 0 },
            IntendsToGather("pig".to_string(), 1),
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let hunter = app.world.entity(hunter_entity);
    assert!(hunter.get::<PathRequest>().is_some());
    assert!(hunter.get::<IntendsToGather>().is_none());
}

#[test]
fn test_hunting_system_finds_closest_pig() {
    // 1. Setup
    let mut app = setup_test_app();

    let far_pig = app.world.spawn((Pig, Position { x: 10, y: 10 })).id();
    let close_pig = app.world.spawn((Pig, Position { x: 3, y: 3 })).id();

    let mut map = app.world.resource_mut::<rust_simulation::map::Map>();
    map.add_entity_to_spatial_map(far_pig, 10, 10);
    map.add_entity_to_spatial_map(close_pig, 3, 3);

    let hunter_entity = app
        .world
        .spawn((
            Position { x: 0, y: 0 },
            IntendsToGather("pig".to_string(), 1),
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    let hunter = app.world.entity(hunter_entity);
    let wants_to_attack = hunter.get::<WantsToAttack>();
    let path_request = hunter.get::<PathRequest>();

    // The hunter should be trying to path to the closest pig.
    assert!(path_request.is_some());
    assert_eq!(path_request.unwrap().goal, (3, 3));
    assert!(wants_to_attack.is_none());
}
