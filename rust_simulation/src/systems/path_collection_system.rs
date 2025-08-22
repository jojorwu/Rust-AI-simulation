use crate::components::path::CurrentPath;
use crate::pathfinding_async::PathfindingResultChannel;
use bevy_ecs::prelude::*;
use log::debug;

pub fn path_collection_system(
    mut commands: Commands,
    channel: Res<PathfindingResultChannel>,
) {
    // Use a while loop with try_recv to drain the channel of all pending results.
    while let Ok(result) = channel.receiver.try_recv() {
        if let Some(path) = result.path {
            debug!(
                "Received path for entity {:?}, adding CurrentPath component.",
                result.entity
            );
            commands.entity(result.entity).insert(CurrentPath {
                nodes: path,
            });
        } else {
            debug!("Received empty path for entity {:?}", result.entity);
            // If no path was found, we could add a component to indicate failure,
            // which could trigger a new goal selection. For now, we just log it.
        }
    }
}
