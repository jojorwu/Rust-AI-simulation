use crate::async_task::{AsyncResult, AsyncResultChannel};
use crate::components::path::CurrentPath;
use bevy_ecs::prelude::*;

pub fn async_result_collection_system(
    mut commands: Commands,
    channel: Res<AsyncResultChannel>,
) {
    while let Ok(result) = channel.receiver.try_recv() {
        match result {
            AsyncResult::Pathfinding(path_result) => {
                if let Some(path) = path_result.path {
                    if let Some(mut entity_commands) = commands.get_entity(path_result.entity) {
                        entity_commands.insert(CurrentPath { nodes: path });
                    }
                }
            }
        }
    }
}
