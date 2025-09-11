use bevy::prelude::*;
use rust_simulation::{
    components::status::Hunger,
    config::Config,
    state::AppState,
    systems::hunger::hunger_system,
};
use bevy::prelude::in_state;

#[test]
fn test_simulation_systems_only_run_in_game() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_state::<AppState>();
    let config = Config::load("data/config.toml").expect("Failed to load config");
    app.insert_resource(config);
    app.add_systems(Update, hunger_system.run_if(in_state(AppState::InGame)));

    // 1. Start in InGame state
    app.world.insert_resource(NextState(Some(AppState::InGame)));
    app.update(); // Enter InGame state

    let entity = app
        .world
        .spawn(Hunger {
            current: 100.0,
            max: 100.0,
        })
        .id();

    // 2. Run app for one tick, hunger should decrease
    app.update();
    let hunger1 = app.world.get::<Hunger>(entity).unwrap().current;
    assert!(hunger1 < 100.0);

    // 3. Switch to MainMenu state
    app.world.insert_resource(NextState(Some(AppState::MainMenu)));
    app.update(); // Enter MainMenu state

    // 4. Run app for another tick, hunger should NOT decrease
    app.update();
    let hunger2 = app.world.get::<Hunger>(entity).unwrap().current;
    assert_eq!(hunger1, hunger2);
}
