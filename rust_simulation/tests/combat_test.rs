use bevy::prelude::*;
use rust_simulation::{
    components::{
        intents::WantsToAttack,
        status::{Damage, Health},
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
    let target = app.world.spawn(Health { current: 50, max: 50 }).id();
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

    // Create target first to get its ID
    let target = app.world.spawn(Health { current: 5, max: 50 }).id();
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
        if let Event::EntityDied(e) = event {
            assert_eq!(*e, target);
            death_event_found = true;
        }
    }
    assert!(death_event_found);
}

#[test]
fn test_health_does_not_go_below_zero() {
    // 1. Setup
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();
    app.add_systems(Update, combat_system);

    // Create target with low health
    let target = app.world.spawn(Health { current: 5, max: 50 }).id();
    // Create attacker that deals more damage than the target has health
    let _attacker = app
        .world
        .spawn((WantsToAttack { target }, Damage(20)))
        .id();

    // 2. Run the system
    app.update();

    // 3. Verify
    let target_health = app
        .world
        .get::<Health>(target)
        .expect("Target should have a Health component");
    // This is the key assertion for the bug.
    // With the bug, health will be -15. After the fix, it should be 0.
    assert_eq!(
        target_health.current,
        0,
        "Target health should be clamped at 0 and not be negative."
    );
}
