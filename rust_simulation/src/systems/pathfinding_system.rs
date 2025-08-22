use crate::components::{ai::MentalMap, path::PathRequest};
use crate::pathfinding;
use crate::pathfinding_async::{PathfindingResult, PathfindingResultChannel};
use bevy_ecs::prelude::*;
use log::debug;
use rayon::spawn;

pub fn pathfinding_system(
    mut commands: Commands,
    query: Query<(Entity, &PathRequest, &MentalMap)>,
    channel: Res<PathfindingResultChannel>,
) {
    for (entity, request, mental_map) in query.iter() {
        // The request is being handled, so remove it immediately.
        commands.entity(entity).remove::<PathRequest>();

        debug!(
            "Spawning pathfinding task for {:?} from {:?} to {:?}",
            entity, request.start, request.goal
        );

        let sender = channel.sender.clone();
        let start = request.start;
        let goal = request.goal;
        // The mental map must be cloned to be sent to the background thread.
        let mental_map_clone = mental_map.0.clone();

        spawn(move || {
            let path = pathfinding::find_path(start, goal, &mental_map_clone);

            let result = PathfindingResult {
                entity,
                path: path.map(|p| p.into()),
            };

            if let Err(e) = sender.send(result) {
                log::error!("Failed to send pathfinding result: {}", e);
            }
        });
    }
}
