use crate::{
    components::{
        status::{Damage, Health},
        WantsToAttack,
    },
    events::Event,
};
use bevy_ecs::prelude::*;
use std::collections::HashSet;

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

    let mut killed_this_tick = HashSet::new();

    for (attacker, target, damage) in to_attack {
        let mut target_was_killed = false;

        if let Ok(mut health) = health_query.get_mut(target) {
            // Only apply damage if the target is actually alive.
            if health.current > 0 {
                health.current -= damage;
                if health.current <= 0 {
                    health.current = 0; // Clamp health at 0 to prevent negative values.
                    target_was_killed = true;
                }
            }
        }

        // If the target was killed by this attack AND we haven't already sent a death event for it,
        // send one now and record it.
        if target_was_killed && !killed_this_tick.contains(&target) {
            event_writer.send(Event::EntityDied(target));
            killed_this_tick.insert(target);
        }

        // The attack intent is always consumed.
        commands.entity(attacker).remove::<WantsToAttack>();
    }
}
