use crate::components::{
    ai::KnownResources,
    intents::IsGathering,
    path::{CurrentPath, PathRequest},
    Inventory, Position, Resource as ResourceComponent,
};
use bevy_ecs::prelude::*;

#[allow(clippy::type_complexity)]
pub fn gathering_system(
    mut commands: Commands,
    mut gatherer_query: Query<
        (
            Entity,
            &mut KnownResources,
            &Position,
            &mut Inventory,
            &mut IsGathering,
        ),
        (Without<CurrentPath>, Without<PathRequest>),
    >,
    mut resource_query: Query<(Entity, &mut ResourceComponent, &Position)>,
) {
    for (entity, mut known_resources, position, mut inventory, mut gathering_state) in
        gatherer_query.iter_mut()
    {
        let target_entity = gathering_state.target;
        let resource_name = gathering_state.resource.clone();
        let target_amount = gathering_state.amount;

        if let Ok((_, mut resource, target_pos)) = resource_query.get_mut(target_entity) {
            let is_adjacent = (position.x.abs_diff(target_pos.x) <= 1)
                && (position.y.abs_diff(target_pos.y) <= 1);

            if is_adjacent {
                if resource.quantity > 0 {
                    resource.quantity -= 1;
                    inventory.add_item(&resource_name, 1);
                    gathering_state.gathered_so_far += 1;

                    if resource.quantity == 0 {
                        commands.entity(target_entity).despawn();
                        // Remove the resource from the agent's known resources.
                        if let Some(positions) = known_resources.0.get_mut(&resource_name) {
                            positions.retain(|&p| p != *target_pos);
                        }
                    }
                }

                // Check if the goal is complete.
                if gathering_state.gathered_so_far >= target_amount || resource.quantity == 0 {
                    // The action is complete, so we can remove the state.
                    commands.entity(entity).remove::<IsGathering>();
                }
            } else {
                // Not adjacent, request a path.
                commands.entity(entity).insert(PathRequest {
                    start: (position.x, position.y),
                    goal: (target_pos.x, target_pos.y),
                });
            }
        } else {
            // The target entity no longer exists, so the gathering state is invalid.
            commands.entity(entity).remove::<IsGathering>();
        }
    }
}
