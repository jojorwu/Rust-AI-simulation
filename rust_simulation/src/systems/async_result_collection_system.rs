use crate::async_task::{AsyncResult, AsyncResultChannel};
use crate::components::{path::CurrentPath, Inventory, Resource as ResourceComponent, Chest};
use crate::events::Event;
use crate::map::{Map, CHUNK_SIZE};
use bevy_ecs::prelude::*;

pub fn async_result_collection_system(
    mut commands: Commands,
    mut inventory_query: Query<&mut Inventory>,
    mut resource_query: Query<&mut ResourceComponent>,
    mut event_writer: EventWriter<Event>,
    map: Res<Map>,
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
            AsyncResult::Crafting(craft_result) => {
                if craft_result.success {
                    if let Ok(mut inventory) = inventory_query.get_mut(craft_result.entity) {
                        if inventory.remove_resources(&craft_result.required_resources) {
                            inventory.add_item(&craft_result.item_name, 1);
                        }
                    }
                }
            }
            AsyncResult::Gathering(gather_result) => {
                if gather_result.gathered_amount > 0 {
                    if let Ok(mut inventory) = inventory_query.get_mut(gather_result.agent_entity) {
                        inventory.add_item(&gather_result.resource_name, gather_result.gathered_amount);
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
            AsyncResult::Building(build_result) => {
                if build_result.success {
                     if let Ok(mut inventory) = inventory_query.get_mut(build_result.builder_entity) {
                        if inventory.remove_resources(&build_result.required_resources) {
                             if let Some(tile) = map.get_tile(build_result.position.x, build_result.position.y) {
                                if tile.tile_type == '.' {
                                    if let Some((chunk_x, chunk_y)) = map.get_chunk_index(build_result.position.x, build_result.position.y) {
                                        let mut chunk = map.chunks[chunk_y][chunk_x].lock().unwrap();
                                        let local_x = (build_result.position.x % CHUNK_SIZE) as usize;
                                        let local_y = (build_result.position.y % CHUNK_SIZE) as usize;
                                        let tile = &mut chunk.tiles[local_y][local_x];

                                        match build_result.structure_name.as_str() {
                                            "chest" => {
                                                commands.spawn((
                                                    build_result.position,
                                                    Chest { inventory: Inventory::new() },
                                                ));
                                                tile.tile_type = 'C';
                                            }
                                            "foundation" => {
                                                tile.tile_type = 'B';
                                                event_writer.send(Event::FoundationBuilt {
                                                    builder: build_result.builder_entity,
                                                    position: build_result.position,
                                                });
                                            }
                                            "wall" => tile.tile_type = '#',
                                            "doorway" => tile.tile_type = 'O',
                                            _ => tile.tile_type = 'X',
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
