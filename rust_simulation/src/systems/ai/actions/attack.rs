use crate::components::{BrainComponent, WantsToAttack, intents::IntendsToAttack};
use bevy_ecs::prelude::*;

pub fn attack_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &IntendsToAttack)>,
) {
    for (entity, mut brain_component, intent) in query.iter_mut() {
        let target_id = intent.0;
        commands.entity(entity).insert(WantsToAttack {
            target: target_id,
        });

        // Attacking is a single-tick action for now.
        brain_component.current_goal = None;
        commands.entity(entity).remove::<IntendsToAttack>();
    }
}
