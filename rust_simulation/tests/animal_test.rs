use bevy::prelude::*;
use rust_simulation::{
    animals::pig::{fleeing_system, Pig, SimpleAi},
    components::{Position, Velocity, WantsToAttack},
};

#[test]
fn test_fleeing_system_removes_wants_to_attack() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, fleeing_system);

    let pig_entity = app
        .world
        .spawn((
            Pig,
            SimpleAi::default(),
            Position { x: 5, y: 5 },
            Velocity { dx: 0, dy: 0 },
        ))
        .id();

    let attacker_entity = app
        .world
        .spawn((
            Position { x: 4, y: 5 },
            WantsToAttack { target: pig_entity },
        ))
        .id();

    app.update();

    assert!(app
        .world
        .get::<WantsToAttack>(attacker_entity)
        .is_none());
}
