use bevy_ecs::prelude::*;
use crate::components::intents::{IntendsToEquip, WantsToEquip};

pub fn equip_action_system(
    mut commands: Commands,
    query: Query<(Entity, &IntendsToEquip)>,
) {
    for (entity, intent) in query.iter() {
        commands.entity(entity)
            .insert(WantsToEquip(intent.0.clone()))
            .remove::<IntendsToEquip>();
    }
}
