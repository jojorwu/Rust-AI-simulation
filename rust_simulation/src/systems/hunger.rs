use crate::{
    components::status::{Health, Hunger},
    config::Config,
    events::Event,
};
use bevy_ecs::prelude::*;

pub fn hunger_system(
    mut query: Query<(Entity, &mut Hunger, Option<&mut Health>)>,
    config: Res<Config>,
    mut event_writer: EventWriter<Event>,
) {
    for (entity, mut hunger, health_option) in query.iter_mut() {
        hunger.current -= config.survival.hunger_rate;
        if hunger.current <= 0.0 {
            hunger.current = 0.0;
            if let Some(mut health) = health_option {
                let old_health = health.current;
                health.current -= config.survival.starvation_damage;
                if old_health > 0 && health.current <= 0 {
                    event_writer.send(Event::EntityDied(entity));
                }
            }
        }
    }
}
