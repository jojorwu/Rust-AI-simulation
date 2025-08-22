use crate::{
    components::{path::PathfindingFailure, BrainComponent},
};
use bevy_ecs::prelude::*;

pub fn handle_pathfinding_failure_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent), With<PathfindingFailure>>,
) {
    for (entity, mut brain) in query.iter_mut() {
        log::debug!(
            "Entity {:?} failed to find a path. Resetting current goal.",
            entity
        );
        brain.current_goal = None;
        brain.goal_commitment_ticks = 0;
        commands.entity(entity).remove::<PathfindingFailure>();
    }
}
