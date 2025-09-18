use bevy::prelude::*;
use rust_simulation::{
    animals::pig::Pig,
    components::{
        intents::IntendsToGather,
        path::PathRequest,
        Position,
    },
    systems::hunting::hunting_system,
};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, hunting_system);
    app
}

#[test]
fn test_hunting_system_path_request() {
    // 1. Setup
    let mut app = setup_test_app();

    let pig_entity = app.world.spawn((Pig, Position { x: 5, y: 5 })).id();
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
