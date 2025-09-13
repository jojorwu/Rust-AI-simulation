use crate::components::{status::{Damage, Health}, WantsToAttack};
use crate::events::Event;
use bevy_ecs::prelude::*;

pub fn combat_system(
    mut commands: Commands,
    query: Query<(Entity, &WantsToAttack, &Damage)>,
    mut health_query: Query<&mut Health>,
    mut event_writer: EventWriter<Event>,
) {
    let mut to_attack = Vec::new();
    for (entity, wants_to_attack, damage) in query.iter() {
        to_attack.push((entity, wants_to_attack.target, damage.0));
    }

    for (attacker, target, damage) in to_attack {
        let mut target_dead = false;
        if let Ok(mut health) = health_query.get_mut(target) {
            if health.current > 0 {
                health.current -= damage;
                if health.current <= 0 {
                    health.current = 0;
                    target_dead = true;
                }
            }
        }

        if target_dead {
            event_writer.send(Event::EntityDied(target));
        }
        commands.entity(attacker).remove::<WantsToAttack>();
    }
}
