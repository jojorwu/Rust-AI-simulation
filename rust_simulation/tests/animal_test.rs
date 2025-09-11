use bevy::prelude::*;
use rust_simulation::{
    animals::pig::{fleeing_system, Pig, SimpleAi},
    components::{Position, Velocity, WantsToAttack},
};

#[test]
fn test_pig_flees_from_same_tile_attacker() {
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
            Position { x: 5, y: 5 },
            WantsToAttack { target: pig_entity },
        ))
        .id();

    // Loop a few times to increase the chance of catching the bug
    for _ in 0..10 {
        app.update();

        let pig_velocity = app.world.get::<Velocity>(pig_entity).unwrap();
        if pig_velocity.dx != 0 || pig_velocity.dy != 0 {
            // The pig has fled, so the test passes
            return;
        }
    }

    // If we get here, the pig never fled
    panic!("Pig did not flee with a non-zero velocity");
}
