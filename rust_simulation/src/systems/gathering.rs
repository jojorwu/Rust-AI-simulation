use crate::components::{
    Inventory, Position, Resource as ResourceComponent,
    ai::KnownResources,
    intents::IntendsToGather,
    path::{CurrentPath, PathRequest},
};
use crate::map::Map;
use bevy_ecs::prelude::*;

pub fn gathering_system(
    mut commands: Commands,
    // Query for agents that intend to gather and are not currently moving.
    mut gatherer_query: Query<
        (
            Entity,
            &KnownResources,
            &Position,
            &mut Inventory,
            &IntendsToGather,
        ),
        (Without<CurrentPath>, Without<PathRequest>),
    >,
    mut resource_query: Query<&mut ResourceComponent>,
    map: Res<Map>,
) {
    for (entity, known_resources, position, mut inventory, intent) in gatherer_query.iter_mut() {
        let resource_name = &intent.0;

        // 1. Find the closest known resource of the desired type from the agent's brain.
        let target_pos = if let Some(known_positions) = known_resources.0.get(resource_name) {
            known_positions
                .iter()
                .min_by_key(|pos| pos.x.abs_diff(position.x) + pos.y.abs_diff(position.y))
                .copied() // We need to copy the Position to use it.
        } else {
            None
        };

        if let Some(target_pos) = target_pos {
            // 2. Check if the agent is adjacent to the target resource.
            let is_adjacent = (position.x.abs_diff(target_pos.x) <= 1)
                && (position.y.abs_diff(target_pos.y) <= 1);

            if is_adjacent {
                // 3. If adjacent, perform the gathering action.
                if let Some(target_entity) = map
                    .get_entities_at(target_pos.x, target_pos.y)
                    .and_then(|v| v.first().copied())
                {
                    if let Ok(mut resource) = resource_query.get_mut(target_entity) {
                        if resource.quantity > 0 {
                            resource.quantity -= 1;
                            inventory.add_item(&resource.name, 1);

                            if resource.quantity == 0 {
                                commands.entity(target_entity).despawn();
                            }
                        }
                        // The action is complete, so we can remove the intent.
                        commands.entity(entity).remove::<IntendsToGather>();
                    } else {
                        // The resource entity at the target position is gone.
                        // The goal has failed. Remove the intent.
                        // The brain should update its known_resources, but that's a bigger task.
                        commands.entity(entity).remove::<IntendsToGather>();
                    }
                } else {
                    // There is no entity at the target position. The goal has failed.
                    commands.entity(entity).remove::<IntendsToGather>();
                }
            } else {
                // 4. If not adjacent, request a path to the target.
                commands.entity(entity).insert(PathRequest {
                    start: (position.x, position.y),
                    goal: (target_pos.x, target_pos.y),
                });
                // The IntendsToGather component remains, as the goal is not yet complete.
            }
        } else {
            // 5. If no resource is known, the goal is impossible. Remove the intent.
            // This will allow the goal selection system to choose a new goal next tick.
            commands.entity(entity).remove::<IntendsToGather>();
        }
    }
}
