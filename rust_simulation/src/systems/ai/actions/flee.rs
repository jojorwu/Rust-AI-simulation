use crate::components::{BrainComponent, Velocity, intents::IntendsToFlee};
use bevy_ecs::prelude::*;
use rand::Rng;

pub fn flee_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent), With<IntendsToFlee>>,
) {
    let mut rng = rand::thread_rng();
    for (entity, mut brain_component) in query.iter_mut() {
        let dx = rng.random_range(-1..=1);
        let dy = rng.random_range(-1..=1);
        commands.entity(entity).insert(Velocity { dx, dy });

        // Fleeing is a single-tick action.
        brain_component.current_goal = None;
        commands.entity(entity).remove::<IntendsToFlee>();
    }
}
