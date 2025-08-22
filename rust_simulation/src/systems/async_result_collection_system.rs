use crate::async_task::{AsyncResult, AsyncResultChannel};
use crate::components::{
    path::{CurrentPath, PathRequest, PathfindingFailure, PathfindingInProgress},
    Inventory, Resource as ResourceComponent,
};
use bevy_ecs::prelude::*;

use crate::events::Event;

pub fn async_result_collection_system(
    mut commands: Commands,
    mut inventory_query: Query<&mut Inventory>,
    mut resource_query: Query<&mut ResourceComponent>,
    mut event_writer: EventWriter<Event>,
    channel: Res<AsyncResultChannel>,
) {
    while let Ok(result) = channel.receiver.try_recv() {
        match result {
            AsyncResult::Pathfinding(path_result) => {
                let mut entity_commands = commands.entity(path_result.entity);
                entity_commands.remove::<(PathRequest, PathfindingInProgress)>();

                if let Some(path) = path_result.path {
                    entity_commands.insert(CurrentPath { nodes: path });
                } else {
                    entity_commands.insert(PathfindingFailure);
                }
            }
            AsyncResult::Crafting(craft_result) => {
                if craft_result.success {
                    if let Ok(mut inventory) = inventory_query.get_mut(craft_result.entity) {
                        if inventory.remove_resources(&craft_result.required_resources) {
                            inventory.add_item(&craft_result.item_name, 1);
                            event_writer.send(Event::ItemCrafted {
                                entity: craft_result.entity,
                                item_name: craft_result.item_name.clone(),
                            });
                        }
                    }
                }
            }
            AsyncResult::Gathering(gather_result) => {
                if gather_result.gathered_amount > 0 {
                    if let Ok(mut inventory) = inventory_query.get_mut(gather_result.agent_entity) {
                        inventory.add_item(&gather_result.resource_name, gather_result.gathered_amount);
                        event_writer.send(Event::ResourceGathered {
                            entity: gather_result.agent_entity,
                            resource: gather_result.resource_name.clone(),
                            quantity: gather_result.gathered_amount,
                        });
                    }
                    if let Ok(mut resource) = resource_query.get_mut(gather_result.resource_entity) {
                        resource.quantity -= gather_result.gathered_amount;
                    }
                }
                if gather_result.despawn_resource {
                    if let Some(mut entity_commands) = commands.get_entity(gather_result.resource_entity) {
                        entity_commands.despawn();
                    }
                }
            }
        }
    }
}
