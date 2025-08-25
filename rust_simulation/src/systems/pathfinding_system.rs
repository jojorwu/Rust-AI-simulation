use crate::async_task::PathfindingResult;
use crate::components::{
    ai::MentalMap,
    path::{PathRequest, PathfindingTask},
};
use crate::pathfinding;
use bevy_ecs::prelude::*;
use bevy_tasks::{AsyncComputeTaskPool, Task};

pub fn pathfinding_system(
    mut commands: Commands,
    query: Query<(Entity, &PathRequest, &MentalMap), Without<PathfindingTask>>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    for (entity, request, mental_map) in query.iter() {
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

        commands.entity(entity).insert(PathfindingTask(task));
    }
}
