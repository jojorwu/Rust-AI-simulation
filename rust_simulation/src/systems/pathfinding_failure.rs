use crate::components::{path::PathfindingFailed, BrainComponent};
use bevy_ecs::prelude::*;
use log::warn;

pub fn pathfinding_failure_system(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&mut BrainComponent>), With<PathfindingFailed>>,
) {
    for (entity, opt_brain) in query.iter_mut() {
        if let Some(mut brain) = opt_brain {
            warn!(
                "Pathfinding failed for entity {:?}, clearing goal.",
                entity
            );
            brain.current_goal = None;
        }
        // Always remove the marker component, regardless of whether the entity had a brain.
        commands.entity(entity).remove::<PathfindingFailed>();
    }
}
