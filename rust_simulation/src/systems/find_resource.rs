use crate::{
    components::{
        ai::KnownResources,
        intents::{IntendsToGather, IsGathering},
        Position,
    },
    map::Map,
};
use bevy_ecs::prelude::*;

pub fn find_resource_system(
    mut commands: Commands,
    query: Query<(Entity, &KnownResources, &Position, &IntendsToGather)>,
    map: Res<Map>,
) {
    for (entity, known_resources, position, intent) in query.iter() {
        let resource_name = &intent.0;
        let amount = intent.1;

        let target_pos = if let Some(known_positions) = known_resources.0.get(resource_name) {
            known_positions
                .iter()
                .min_by_key(|pos| pos.x.abs_diff(position.x) + pos.y.abs_diff(position.y))
                .copied()
        } else {
            None
        };

        if let Some(target_pos) = target_pos {
            if let Some(target_entity) = map
                .get_entities_at(target_pos.x, target_pos.y)
                .and_then(|v| v.first().copied())
            {
                // We found a target, so remove the general intent and add a specific gathering state.
                commands.entity(entity).remove::<IntendsToGather>();
                commands.entity(entity).insert(IsGathering {
                    target: target_entity,
                    resource: resource_name.clone(),
                    amount,
                    gathered_so_far: 0,
                });
            }
        }
        // If no resource is found, the IntendsToGather component remains, and the
        // AI will eventually choose a new goal (likely Explore).
    }
}
