use crate::{
    components::{
        intents::{IntendsToGather, IsGathering},
        Position,
    },
    spatial::{SpatialIndex, SpatialPoint},
};
use bevy_ecs::prelude::*;

pub fn find_resource_system(
    mut commands: Commands,
    spatial_index: Res<SpatialIndex>,
    query: Query<(Entity, &Position, &IntendsToGather)>,
) {
    for (entity, position, intent) in query.iter() {
        let resource_name = &intent.0;
        let amount = intent.1;

        let query_point = SpatialPoint {
            x: position.x as i32,
            y: position.y as i32,
            entity: Entity::from_raw(0), // Dummy entity
        };
        if let Some(closest_resource) = spatial_index.resources.nearest_neighbor(&query_point)
        {
            // We found a target, so remove the general intent and add a specific gathering state.
            commands.entity(entity).remove::<IntendsToGather>();
            commands.entity(entity).insert(IsGathering {
                target: closest_resource.entity,
                resource: resource_name.clone(),
                amount,
            });
        }
        // If no resource is found, the IntendsToGather component remains, and the
        // AI will eventually choose a new goal (likely Explore).
    }
}
