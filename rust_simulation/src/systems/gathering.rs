use crate::components::{
    ai::KnownResources,
    intents::IntendsToGatherFrom,
    path::{CurrentPath, PathRequest},
    Inventory, Position, Resource as ResourceComponent,
};
use bevy_ecs::prelude::*;

pub fn gathering_system(
    mut commands: Commands,
    mut gatherer_query: Query<
        (
            Entity,
            &mut KnownResources,
            &Position,
            &mut Inventory,
            &IntendsToGatherFrom,
        ),
        (Without<CurrentPath>, Without<PathRequest>),
    >,
    mut resource_query: Query<(Entity, &mut ResourceComponent, &Position)>,
) {
    for (entity, mut known_resources, position, mut inventory, intent) in gatherer_query.iter_mut()
    {
        let target_entity = intent.0;

        if let Ok((_, mut resource, target_pos)) = resource_query.get_mut(target_entity) {
            let is_adjacent = (position.x.abs_diff(target_pos.x) <= 1)
                && (position.y.abs_diff(target_pos.y) <= 1);

            if is_adjacent {
                if resource.quantity > 0 {
                    resource.quantity -= 1;
                    inventory.add_item(&resource.name, 1);

                    if resource.quantity == 0 {
                        commands.entity(target_entity).despawn();
                        // Remove the resource from the agent's known resources.
                        if let Some(positions) = known_resources.0.get_mut(&resource.name) {
                            positions.retain(|&p| p != *target_pos);
                        }
                    }
                }
                // The action is complete, so we can remove the intent.
                commands.entity(entity).remove::<IntendsToGatherFrom>();
            } else {
                // Not adjacent, request a path.
                commands.entity(entity).insert(PathRequest {
                    start: (position.x, position.y),
                    goal: (target_pos.x, target_pos.y),
                });
            }
        } else {
            // The target entity does not have a ResourceComponent, so it's not a resource.
            // This should not happen if the find_resource_system is working correctly.
            commands.entity(entity).remove::<IntendsToGatherFrom>();
        }
    }
}
