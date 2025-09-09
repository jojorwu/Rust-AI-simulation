use crate::async_task::PathfindingResult;
use crate::components::{
    ai::MentalMap,
    path::{PathRequest, PathfindingTask},
};
use crate::pathfinding;
use bevy_ecs::prelude::*;
use bevy_tasks::{AsyncComputeTaskPool, Task};

/// A system that spawns asynchronous pathfinding tasks for entities with a `PathRequest`.
///
/// This system looks for any entity that has a `PathRequest` component and creates a
/// background task to calculate the path using the A* algorithm. It also handles
/// cancelling any pre-existing pathfinding task for that entity.
pub fn pathfinding_system(
    mut commands: Commands,
    mut query: Query<(Entity, &PathRequest, &MentalMap, Option<&mut PathfindingTask>)>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    for (entity, request, mental_map, existing_task) in query.iter_mut() {
        // --- Task Cancellation ---
        // If a new PathRequest is added to an entity that already has a PathfindingTask,
        // we assume the old one is obsolete. We detach it, which drops the task handle
        // and allows the scheduler to stop working on it.
        if let Some(mut task) = existing_task {
            task.0.detach();
        }

        let start = request.start;
        let goal = request.goal;
        let mental_map_clone = mental_map.clone();

        let task: Task<PathfindingResult> = task_pool.spawn(async move {
            let path = pathfinding::find_path(start, goal, &mental_map_clone.0);

            PathfindingResult {
                entity,
                path: path.map(|p| p.into()),
            }
        });

        // Insert the new task, replacing the old one if it existed.
        commands.entity(entity).insert(PathfindingTask(task));
        // Remove the request component so we don't immediately try to pathfind again.
        commands.entity(entity).remove::<PathRequest>();
    }
}
