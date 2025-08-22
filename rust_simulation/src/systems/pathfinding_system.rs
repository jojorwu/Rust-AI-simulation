use bevy_ecs::prelude::*;
use crate::components::{
    path::{PathRequest, CurrentPath},
    BrainComponent,
};
use crate::pathfinding;
use std::collections::VecDeque;
use log::debug;

pub fn pathfinding_system(
    mut commands: Commands,
    // We query for BrainComponent to get access to the agent's mental_map
    query: Query<(Entity, &PathRequest, &BrainComponent)>,
) {
    for (entity, request, brain) in query.iter() {
        debug!("Processing path request for {:?} from {:?} to {:?}", entity, request.start, request.goal);
        let path = pathfinding::find_path(
            request.start,
            request.goal,
            &brain.mental_map,
        );

        if let Some(nodes) = path {
            // Path found, add the CurrentPath component
            debug!("Path found for {:?}, adding CurrentPath component.", entity);
            commands.entity(entity).insert(CurrentPath {
                nodes: VecDeque::from(nodes),
            });
        } else {
            debug!("No path found for {:?}", entity);
        }

        // Always remove the request component after processing
        commands.entity(entity).remove::<PathRequest>();
    }
}
