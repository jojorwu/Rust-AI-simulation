use crate::components::{Health, WantsToAttack};
use crate::events::Event;
use bevy_ecs::prelude::*;

pub fn combat_system(
    mut commands: Commands,
    query: Query<(Entity, &WantsToAttack)>,
    mut health_query: Query<&mut Health>,
    mut event_writer: EventWriter<Event>,
) {
    let mut to_attack = Vec::new();
    for (entity, wants_to_attack) in query.iter() {
        to_attack.push((entity, wants_to_attack.target));
    }

    for (attacker, target) in to_attack {
        let damage = 10; // Placeholder
        let mut target_dead = false;
        if let Ok(mut health) = health_query.get_mut(target) {
            health.current -= damage;
            if health.current <= 0 {
                target_dead = true;
            }
        }

        if target_dead {
            event_writer.send(Event::EntityDied(target));
        }
        commands.entity(attacker).remove::<WantsToAttack>();
    }
}
