use crate::components::path::{CurrentPath, PathRequest, PathfindingFailed, PathfindingTask};
use bevy_ecs::prelude::*;
use bevy_tasks::futures_lite::future;

/// A system that checks for completed pathfinding tasks and handles their results.
///
/// This system polls all active `PathfindingTask` components. If a task is finished,
/// it removes the task component and attaches either a `CurrentPath` component on
/// success or a `PathfindingFailed` component on failure.
pub fn pathfinding_completion_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut PathfindingTask)>,
) {
    for (entity, mut task) in query.iter_mut() {
        // Poll the task to see if it has completed. `poll_once` is non-blocking.
        if let Some(path_result) = future::block_on(future::poll_once(&mut task.0)) {
            // The task is finished, so we can remove the task and request components.
            // The `PathRequest` is removed here as well as in the spawning system
            // to handle cases where a request might be added and completed in the same frame.
            commands.entity(entity).remove::<PathfindingTask>();
            commands.entity(entity).remove::<PathRequest>();

            if let Some(path) = path_result.path {
                commands.entity(entity).insert(CurrentPath {
                    nodes: path,
                    stuck_timer: 0,
                });
            } else {
                commands.entity(entity).insert(PathfindingFailed);
            }
        }
    }
}
