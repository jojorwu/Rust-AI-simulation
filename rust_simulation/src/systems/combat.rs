use crate::components::{
    intents::WantsToAttack,
    status::{Damage, Health},
    Position,
};
use crate::events::Event;
use bevy_ecs::prelude::*;

pub fn combat_system(
    mut commands: Commands,
    query: Query<(Entity, &WantsToAttack, &Damage)>,
    mut health_query: Query<(&mut Health, &Position)>,
    mut event_writer: EventWriter<Event>,
) {
    let mut to_attack = Vec::new();
    for (entity, wants_to_attack, damage) in query.iter() {
        to_attack.push((entity, wants_to_attack.target, damage.0));
    }

    for (attacker, target, damage) in to_attack {
        let mut target_dead = false;
        if let Ok((mut health, position)) = health_query.get_mut(target) {
            health.current -= damage;
            if health.current <= 0 {
                target_dead = true;
                event_writer.send(Event::EntityDied {
                    entity: target,
                    position: *position,
                });
            }
            // The attack was successful, so remove the intent.
            commands.entity(attacker).remove::<WantsToAttack>();
        }
        // If the target is invalid, the intent is not removed.
    }
}
