use crate::components::status::{Health, Hunger};
use crate::config::Config;
use bevy_ecs::prelude::*;
use rayon::prelude::*;

pub fn hunger_system(mut query: Query<(&mut Hunger, Option<&mut Health>)>, config: Res<Config>) {
    query.par_iter_mut().for_each(|(mut hunger, health_option)| {
        hunger.current -= config.survival.hunger_rate;
        if hunger.current <= 0.0 {
            hunger.current = 0.0;
            if let Some(mut health) = health_option {
                health.current -= config.survival.starvation_damage;
            }
        }
    });
}
