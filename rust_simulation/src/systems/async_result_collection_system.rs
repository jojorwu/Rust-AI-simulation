use crate::async_task::{AsyncResult, AsyncResultChannel};
use crate::components::{path::CurrentPath, Inventory};
use bevy_ecs::prelude::*;
use log::debug;

pub fn async_result_collection_system(
    mut commands: Commands,
    mut query: Query<&mut Inventory>,
    channel: Res<AsyncResultChannel>,
) {
    // Use a while loop with try_recv to drain the channel of all pending results.
    while let Ok(result) = channel.receiver.try_recv() {
        match result {
            AsyncResult::Pathfinding(path_result) => {
                if let Some(path) = path_result.path {
                    debug!(
                        "Received path for entity {:?}, adding CurrentPath component.",
                        path_result.entity
                    );
                    if let Some(mut entity_commands) = commands.get_entity(path_result.entity) {
                        entity_commands.insert(CurrentPath { nodes: path });
                    }
                } else {
                    debug!("Received empty path for entity {:?}", path_result.entity);
                    // If no path was found, we could add a component to indicate failure,
                    // which could trigger a new goal selection. For now, we just log it.
                }
            }
            AsyncResult::Crafting(craft_result) => {
                if craft_result.success {
                    if let Ok(mut inventory) = query.get_mut(craft_result.entity) {
                        debug!(
                            "Applying successful craft for entity {:?}: {}",
                            craft_result.entity, craft_result.item_name
                        );
                        if inventory.remove_resources(&craft_result.required_resources) {
                            inventory.add_item(&craft_result.item_name, 1);
                        } else {
                            // This could happen if resources were used by another action
                            // between task dispatch and result collection.
                            log::warn!(
                                "Crafting for {:?} failed post-check, resources no longer available.",
                                craft_result.entity
                            );
                        }
                    }
                }
            }
        }
    }
}
