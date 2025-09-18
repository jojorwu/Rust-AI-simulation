use bevy::prelude::*;
use rust_simulation::{
    animals::pig::Pig,
    components::{
        intents::{IntendsToExplore, IntendsToGather},
        Position,
    },
    systems::hunting::hunting_system,
};

#[test]
fn test_hunting_system_no_pigs_should_explore() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // No pigs are spawned in the world.

    let hunter_entity = app
        .world
        .spawn((
            Position { x: 0, y: 0 },
            IntendsToGather("pig".to_string(), 1),
        ))
        .id();

    app.add_systems(Update, hunting_system);
    app.update();

    let hunter = app.world.entity(hunter_entity);
    assert!(
        hunter.get::<IntendsToExplore>().is_some(),
        "Hunter should have IntendsToExplore component when no pigs are found"
    );
    assert!(
        hunter.get::<IntendsToGather>().is_none(),
        "Hunter should no longer have IntendsToGather component"
    );
}

#[test]
fn test_hunting_system_with_pig_far_away_should_pathfind() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.world.spawn((Pig, Position { x: 10, y: 10 }));

    let hunter_entity = app
        .world
        .spawn((
            Position { x: 0, y: 0 },
            IntendsToGather("pig".to_string(), 1),
        ))
        .id();

    app.add_systems(Update, hunting_system);
    app.update();

    let hunter = app.world.entity(hunter_entity);
    assert!(
        hunter
            .get::<rust_simulation::components::path::PathRequest>()
            .is_some(),
        "Hunter should have PathRequest component when pig is far away"
    );
}
