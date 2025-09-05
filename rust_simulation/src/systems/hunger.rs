use crate::components::status::{Health, Hunger};
use crate::config::Config;
use bevy_ecs::prelude::*;

pub fn hunger_system(mut query: Query<(&mut Hunger, Option<&mut Health>)>, config: Res<Config>) {
    for (mut hunger, health_option) in query.iter_mut() {
        hunger.current -= config.survival.hunger_rate;
        if hunger.current <= 0.0 {
            hunger.current = 0.0;
            if let Some(mut health) = health_option {
                health.current -= config.survival.starvation_damage;
            }
        }
    }
}
