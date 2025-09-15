use bevy::prelude::*;
use rust_simulation::{
    components::status::{Health, Hunger},
    config::{Ai, Config, DayNightCycle, Goals, MapSettings, PigSettings, PlayerSettings, QLearning, SurvivalSettings, TrainingSettings},
    events::Event,
    systems::hunger::hunger_system,
};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();

    // Add a mock config for the hunger system to use
    let config = Config {
        survival: SurvivalSettings {
            hunger_rate: 10.0,
            starvation_damage: 5,
            meat_hunger_value: 25.0,
        },
        day_night_cycle: DayNightCycle {
            day_length: 1000,
            night_length: 500,
        },
        map_settings: MapSettings {
            width: 100,
            height: 100,
            seed: Some(0),
        },
        player_settings: PlayerSettings { num_players: 1 },
        pig_settings: PigSettings { num_pigs: 0 },
        training_settings: TrainingSettings { episodes: 0, max_steps_per_episode: 0 },
        ai: Ai {
            opportunistic_commitment_threshold: 5,
            valuable_resources: vec![],
            defense_radius: 10,
            critical_health_ratio: 0.25,
            standard_health_ratio: 0.75,
            vision_radius: 10,
            q_learning: QLearning {
                learning_rate: 0.1,
                discount_factor: 0.9,
                epsilon: 0.1,
                epsilon_decay: 0.995,
                min_epsilon: 0.01,
            },
            goals: Goals {
                reward: 10.0,
                penalty: -10.0,
                build_bonus: 50.0,
                gather_threshold: 5,
                commitment_ticks: 10,
                threat_commitment_ticks: 5,
            },
        },
    };
    app.insert_resource(config);

    app.add_systems(Update, hunger_system);
    app
}

#[test]
fn test_starvation_sends_death_event() {
    // 1. Setup
    let mut app = setup_test_app();

    // Create an entity with low health and zero hunger
    let starving_entity = app
        .world
        .spawn((
            Hunger {
                current: 0.0,
                max: 100.0,
            },
            Health { current: 3, max: 100 },
        ))
        .id();

    // 2. Run system
    app.update();

    // 3. Verify
    // The entity should have taken 5 starvation damage, bringing health to -2.
    // A death event should have been fired.
    let events = app.world.resource::<Events<Event>>();
    let mut reader = events.get_reader();
    let mut death_event_found = false;
    for event in reader.read(events) {
        if let Event::EntityDied(e) = event {
            assert_eq!(*e, starving_entity);
            death_event_found = true;
        }
    }

    assert!(
        death_event_found,
        "An EntityDied event should be sent when an entity starves to death"
    );
}
