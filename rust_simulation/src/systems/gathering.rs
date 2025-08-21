use crate::components::{Inventory, Position, Resource as ResourceComponent, WantsToGather};
use bevy_ecs::prelude::*;

pub fn gathering_system(
    mut commands: Commands,
    mut gatherer_query: Query<(Entity, &Position, &mut Inventory, &WantsToGather)>,
    mut resource_query: Query<(&mut ResourceComponent, &Position)>,
) {
    for (gatherer_entity, gatherer_pos, mut inventory, wants) in gatherer_query.iter_mut() {
        if let Ok((mut resource, resource_pos)) = resource_query.get_mut(wants.target) {
            let dx = (gatherer_pos.x as i32 - resource_pos.x as i32).abs();
            let dy = (gatherer_pos.y as i32 - resource_pos.y as i32).abs();

            // Check if the gatherer is adjacent to the resource
            if dx <= 1 && dy <= 1 {
                if resource.quantity > 0 {
                    resource.quantity -= 1;
                    inventory.add_item(&resource.name, 1);

                    // If the resource is depleted, despawn it
                    if resource.quantity == 0 {
                        commands.entity(wants.target).despawn();
                    }
                }
            }
        }
        // The entity has attempted to gather, so remove the component.
        // In a more complex system, this might only be removed on success.
        commands.entity(gatherer_entity).remove::<WantsToGather>();
    }
}
