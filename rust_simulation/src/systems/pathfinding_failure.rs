use crate::components::{path::PathfindingFailed, BrainComponent};
use bevy_ecs::prelude::*;

pub fn pathfinding_failure_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent), With<PathfindingFailed>>,
) {
    for (entity, mut brain) in query.iter_mut() {
        println!("Pathfinding failed for entity {:?}, clearing goal.", entity);
        brain.current_goal = None;
        commands.entity(entity).remove::<PathfindingFailed>();
    }
}
