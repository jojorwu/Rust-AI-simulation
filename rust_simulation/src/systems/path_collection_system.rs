use crate::components::path::CurrentPath;
use crate::async_task::{AsyncResult, AsyncResultChannel};
use bevy_ecs::prelude::*;
use log::debug;

pub fn path_collection_system(mut commands: Commands, channel: Res<AsyncResultChannel>) {
    while let Ok(async_result) = channel.receiver.try_recv() {
        match async_result {
            AsyncResult::Pathfinding(result) => {
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
                }
            }
        }
    }
}
