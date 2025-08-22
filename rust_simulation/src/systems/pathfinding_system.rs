use crate::async_task::{AsyncResult, AsyncResultChannel, PathfindingResult};
use crate::components::{
    ai::MentalMap,
    path::{PathRequest, PathfindingInProgress},
};
use crate::pathfinding;
use bevy_ecs::prelude::*;
use rayon::spawn;

pub fn pathfinding_dispatcher_system(
    mut commands: Commands,
    query: Query<(Entity, &PathRequest, &MentalMap), Without<PathfindingInProgress>>,
    channel: Res<AsyncResultChannel>,
) {
    for (entity, request, mental_map) in query.iter() {
        commands.entity(entity).insert(PathfindingInProgress);

        let sender = channel.sender.clone();
        let start = request.start;
        let goal = request.goal;
        let mental_map_clone = mental_map.clone();

        spawn(move || {
            let path = pathfinding::find_path(start, goal, &mental_map_clone.0);

            let result = PathfindingResult {
                entity,
                path: path.map(|p| p.into()),
            };

            if let Err(e) = sender.send(AsyncResult::Pathfinding(result)) {
                log::error!("Failed to send pathfinding result: {}", e);
            }
        });
    }
}
