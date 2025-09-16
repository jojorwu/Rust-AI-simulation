use bevy::prelude::*;
use rust_simulation::{
    components::{
        intents::WantsToAttack,
        status::{Damage, Health},
        Position,
    },
    events::Event,
    systems::combat::combat_system,
};

#[test]
fn test_combat_system_applies_damage() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();
    app.add_systems(Update, combat_system);

    // Create target first to get its ID
    let target = app
        .world
        .spawn((Health { current: 50, max: 50 }, Position { x: 1, y: 1 }))
        .id();
    // Create attacker with the correct target ID
    let attacker = app
        .world
        .spawn((WantsToAttack { target }, Damage(10)))
        .id();

    // 2. Run the system
    app.update();

    // 3. Verify
    // Attacker should no longer want to attack
    assert!(app.world.get::<WantsToAttack>(attacker).is_none());

    // Target's health should be reduced
    let target_health = app
        .world
        .get::<Health>(target)
        .expect("Target should have a Health component");
    assert_eq!(target_health.current, 40); // 50 - 10 damage

    // No death event should be sent
    let events = app.world.resource::<Events<Event>>();
    assert_eq!(events.len(), 0);
}

#[test]
fn test_combat_system_handles_death() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();
    app.add_systems(Update, combat_system);

    let target_pos = Position { x: 2, y: 2 };
    // Create target first to get its ID
    let target = app
        .world
        .spawn((Health { current: 5, max: 50 }, target_pos))
        .id();
    // Create attacker with the correct target ID
    let _attacker = app
        .world
        .spawn((WantsToAttack { target }, Damage(10)))
        .id();

    // 2. Run the system
    app.update();

    // 3. Verify
    // Target's health should be <= 0
    let target_health = app
        .world
        .get::<Health>(target)
        .expect("Target should have a Health component");
    assert!(target_health.current <= 0);

    // A death event should have been sent
    let events = app.world.resource::<Events<Event>>();
    let mut reader = events.get_reader();
    let mut death_event_found = false;
    for event in reader.read(events) {
        if let Event::EntityDied { entity, position } = event {
            assert_eq!(*entity, target);
            assert_eq!(*position, target_pos);
            death_event_found = true;
        }
    }
    assert!(death_event_found);
}

#[test]
fn test_combat_intent_persists_if_target_is_invalid() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();
    app.add_systems(Update, combat_system);

    // Create an invalid target entity
    let invalid_target = Entity::from_raw(999);

    // Create attacker with the invalid target ID
    let attacker = app
        .world
        .spawn((WantsToAttack { target: invalid_target }, Damage(10)))
        .id();

    // 2. Run the system
    app.update();

    // 3. Verify
    // Attacker should still want to attack because the target was invalid
    assert!(app.world.get::<WantsToAttack>(attacker).is_some());
}
