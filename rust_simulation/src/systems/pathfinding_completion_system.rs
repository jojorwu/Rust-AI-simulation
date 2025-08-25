use crate::components::path::{CurrentPath, PathRequest, PathfindingFailed, PathfindingTask};
use bevy_ecs::prelude::*;
use bevy_tasks::futures_lite::future;

pub fn pathfinding_completion_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut PathfindingTask)>,
) {
    for (entity, mut task) in query.iter_mut() {
        if let Some(path_result) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity).remove::<PathfindingTask>();
            commands.entity(entity).remove::<PathRequest>();

            if let Some(path) = path_result.path {
                commands.entity(entity).insert(CurrentPath { nodes: path });
            } else {
                commands.entity(entity).insert(PathfindingFailed);
            }
        }
    }
}
