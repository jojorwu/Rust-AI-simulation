use criterion::{criterion_group, criterion_main, Criterion};
use rust_simulation::systems::ai::goal_selection::goal_selection_system;
use rust_simulation::systems::ai::q_learning::update_q_table_system;
use bevy::prelude::*;
use rust_simulation::components::{BrainComponent, ai::GoalQTable, Health, Inventory, ai::KnownResources, ai::PlayerMemories};
use rust_simulation::config::Config;
use rust_simulation::map::Map;
use rust_simulation::player::Player;
use rust_simulation::IsDay;
use rust_simulation::events::Event;
use rust_simulation::recipes::RecipeManager;
use std::collections::HashMap;
use std::sync::Arc;

fn setup_app(num_agents: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<Event>();

    let config = Config::load("/app/rust_simulation/data/config.toml").unwrap();

    let map = Map::new(
        config.map_settings.width,
        config.map_settings.height,
        "/app/rust_simulation/data/biomes.json",
        "/app/rust_simulation/data/resources.json",
        config.map_settings.seed,
    )
    .unwrap();

    let recipe_manager = Arc::new(RecipeManager::new("/app/rust_simulation/data/recipes.json").unwrap());

    app.insert_resource(map);
    app.insert_resource(config.clone());
    app.insert_resource(IsDay(true));

    for i in 0..num_agents {
        app.world.spawn((
            Player::new(i, 100, 100),
            BrainComponent::new(
                recipe_manager.clone(),
                config.ai.q_learning.learning_rate,
                config.ai.q_learning.discount_factor,
                config.ai.q_learning.epsilon,
            ),
            Health { current: 100, max: 100 },
            Inventory::new(),
            KnownResources(HashMap::new()),
            PlayerMemories(HashMap::new()),
            GoalQTable(HashMap::new()),
        ));
    }

    app
}

fn benchmark_ai_systems(c: &mut Criterion) {
    let mut group = c.benchmark_group("AI Systems");
    for num_agents in [100, 500, 1000].iter() {
        group.bench_function(format!("{} agents", num_agents), |b| {
            b.iter_with_setup(
                || {
                    let mut app = setup_app(*num_agents);
                    app.add_systems(Update, (goal_selection_system, update_q_table_system));
                    app
                },
                |mut app| {
                    app.update();
                },
            );
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_ai_systems);
criterion_main!(benches);
