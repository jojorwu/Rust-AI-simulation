use crate::components::animal::Hunger;
use crate::components::Health;
use bevy_ecs::prelude::*;

const HUNGER_RATE: f32 = 0.01;
const STARVATION_DAMAGE: i32 = 1;

pub fn hunger_system(mut query: Query<(&mut Hunger, Option<&mut Health>)>) {
    for (mut hunger, health_option) in query.iter_mut() {
        hunger.current -= HUNGER_RATE;
        if hunger.current <= 0.0 {
            hunger.current = 0.0;
            if let Some(mut health) = health_option {
                health.current -= STARVATION_DAMAGE;
            }
        }
    }
}
