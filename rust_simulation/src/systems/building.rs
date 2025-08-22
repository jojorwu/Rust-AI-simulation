use crate::async_task::{AsyncResult, AsyncResultChannel, BuildingResult};
use crate::components::{intents::IntendsToBuild, Inventory, Position};
use crate::RecipeManagerResource;
use bevy_ecs::prelude::*;
use rayon::spawn;

pub fn building_dispatcher_system(
    mut commands: Commands,
    builder_query: Query<(Entity, &Position, &Inventory, &IntendsToBuild)>,
    recipe_manager: Res<RecipeManagerResource>,
    channel: Res<AsyncResultChannel>,
) {
    for (builder_entity, pos, inventory, intends_to_build) in builder_query.iter() {
        commands.entity(builder_entity).remove::<IntendsToBuild>();

        let task = crate::async_task::BuildingTask {
            builder_entity,
            position: *pos,
            inventory: inventory.clone(),
            structure_name: intends_to_build.0.clone(),
            recipe_manager: recipe_manager.0.clone(),
        };

        let sender = channel.sender.clone();

        spawn(move || {
            let result = building_worker(task);
            if let Err(e) = sender.send(AsyncResult::Building(result)) {
                log::error!("Failed to send building result: {}", e);
            }
        });
    }
}

// The worker only checks for resources. The check for whether the tile is
// valid is moved to the collection system.
fn building_worker(
    task: crate::async_task::BuildingTask,
) -> BuildingResult {
    let required = task
        .recipe_manager
        .get_required_resources(&task.structure_name, 1);

    let success = task.inventory.has_resources(&required);

    BuildingResult {
        builder_entity: task.builder_entity,
        position: task.position,
        structure_name: task.structure_name,
        required_resources: required,
        success,
    }
}
