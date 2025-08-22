use crate::async_task::{AsyncResult, AsyncResultChannel, GatheringResult};
use crate::components::{
    ai::KnownResources,
    intents::IntendsToGather,
    path::{CurrentPath, PathRequest},
    Position, Resource as ResourceComponent,
};
use bevy_ecs::prelude::*;
use rayon::spawn;

/// This system handles agents that need to move towards a resource to gather it.
pub fn gathering_movement_system(
    mut commands: Commands,
    gatherer_query: Query<
        (Entity, &KnownResources, &Position, &IntendsToGather),
        (Without<CurrentPath>, Without<PathRequest>),
    >,
) {
    for (entity, known_resources, position, intent) in gatherer_query.iter() {
        let resource_name = &intent.0;

        let target_pos =
            if let Some(known_positions) = known_resources.0.get(resource_name) {
                known_positions
                    .iter()
                    .min_by_key(|pos| pos.x.abs_diff(position.x) + pos.y.abs_diff(position.y))
                    .copied()
            } else {
                None
            };

        if let Some(target_pos) = target_pos {
            let is_adjacent = (position.x.abs_diff(target_pos.x) <= 1)
                && (position.y.abs_diff(target_pos.y) <= 1);

            if !is_adjacent {
                commands.entity(entity).insert(PathRequest {
                    start: (position.x, position.y),
                    goal: (target_pos.x, target_pos.y),
                });
            }
        } else {
            commands.entity(entity).remove::<IntendsToGather>();
        }
    }
}

/// This system dispatches gathering tasks for agents that are adjacent to a resource.
pub fn gathering_dispatcher_system(
    mut commands: Commands,
    gatherer_query: Query<(Entity, &Position, &IntendsToGather)>,
    resource_query: Query<(Entity, &Position, &ResourceComponent)>,
    channel: Res<AsyncResultChannel>,
) {
    for (agent_entity, agent_pos, intent) in gatherer_query.iter() {
        for (resource_entity, resource_pos, resource) in resource_query.iter() {
            let is_adjacent = (agent_pos.x.abs_diff(resource_pos.x) <= 1)
                && (agent_pos.y.abs_diff(resource_pos.y) <= 1);

            if is_adjacent && resource.name == intent.0 {
                commands.entity(agent_entity).remove::<IntendsToGather>();

                let task = crate::async_task::GatheringTask {
                    agent_entity,
                    resource_entity,
                    resource_name: resource.name.clone(),
                    resource_quantity: resource.quantity,
                };

                let sender = channel.sender.clone();
                spawn(move || {
                    let result = gathering_worker(task);
                    if let Err(e) = sender.send(AsyncResult::Gathering(result)) {
                        log::error!("Failed to send gathering result: {}", e);
                    }
                });
                break;
            }
        }
    }
}

fn gathering_worker(task: crate::async_task::GatheringTask) -> GatheringResult {
    let gathered_amount = if task.resource_quantity > 0 { 1 } else { 0 };
    let despawn_resource = task.resource_quantity <= 1;

    GatheringResult {
        agent_entity: task.agent_entity,
        resource_entity: task.resource_entity,
        resource_name: task.resource_name,
        gathered_amount,
        despawn_resource,
    }
}
