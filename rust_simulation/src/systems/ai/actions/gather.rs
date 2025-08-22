use bevy_ecs::prelude::*;
use crate::components::{
    intents::IntendsToGather, path::{CurrentPath, PathRequest}, BrainComponent, Position, WantsToGather
};
use crate::map::Map;

pub fn gather_action_system(
    mut commands: Commands,
    // This query now finds entities that intend to gather, but are not currently pathing.
    query: Query<(Entity, &BrainComponent, &Position, &IntendsToGather), (Without<CurrentPath>, Without<PathRequest>)>,
    map: Res<Map>,
) {
    for (entity, brain, position, intent) in query.iter() {
        let resource_name = &intent.0;

        // Find the closest known resource of the desired type.
        let target_pos = if let Some(known_positions) = brain.known_resources.get(resource_name) {
            known_positions
                .iter()
                .min_by_key(|pos| pos.x.abs_diff(position.x) + pos.y.abs_diff(position.y))
        } else {
            None // We don't know where any of this resource is.
        };

        if let Some(target_pos) = target_pos {
            let is_adjacent = (position.x.abs_diff(target_pos.x) <= 1) && (position.y.abs_diff(target_pos.y) <= 1);

            if is_adjacent {
                // If adjacent, find the actual resource entity and add WantsToGather.
                if let Some(target_entity) = map.get_entities_at(target_pos.x, target_pos.y).and_then(|v| v.first().copied()) {
                    commands.entity(entity).insert(WantsToGather { target: target_entity });
                }
                // Whether we found the entity or not, the intent is handled.
                commands.entity(entity).remove::<IntendsToGather>();
            } else {
                // If not adjacent, request a path.
                commands.entity(entity).insert(PathRequest {
                    start: (position.x, position.y),
                    goal: (target_pos.x, target_pos.y),
                });
                // The intent remains until we are adjacent.
            }
        } else {
            // Cannot find resource, so the goal is impossible. Remove intent.
            commands.entity(entity).remove::<IntendsToGather>();
            // In a real scenario, we might want to trigger another goal like Explore.
            // For now, the goal selection will just run again next tick.
        }
    }
}
