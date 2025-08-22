use crate::async_task::{AsyncResult, AsyncResultChannel, CraftingResult};
use crate::components::{Inventory, WantsToCraft};
use crate::RecipeManagerResource;
use bevy_ecs::prelude::*;
use rayon::spawn;

pub fn crafting_dispatcher_system(
    mut commands: Commands,
    query: Query<(Entity, &Inventory, &WantsToCraft)>,
    recipe_manager: Res<RecipeManagerResource>,
    channel: Res<AsyncResultChannel>,
) {
    for (entity, inventory, wants_to_craft) in query.iter() {
        commands.entity(entity).remove::<WantsToCraft>();

        let task = crate::async_task::CraftingTask {
            entity,
            item_name: wants_to_craft.item_name.clone(),
            inventory: inventory.clone(),
            recipe_manager: recipe_manager.0.clone(),
        };

        let sender = channel.sender.clone();

        spawn(move || {
            let result = craft_worker(task);
            if let Err(e) = sender.send(AsyncResult::Crafting(result)) {
                log::error!("Failed to send crafting result: {}", e);
            }
        });
    }
}

fn craft_worker(task: crate::async_task::CraftingTask) -> CraftingResult {
    let required_resources =
        task.recipe_manager
            .get_required_resources(&task.item_name, 1);
    let success = task.inventory.has_resources(&required_resources);

    CraftingResult {
        entity: task.entity,
        item_name: task.item_name,
        required_resources,
        success,
    }
}
