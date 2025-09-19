use crate::components::intents::WantsToCraft;
use crate::components::Inventory;
use crate::RecipeManagerResource;
use bevy_ecs::prelude::*;
use log::error;

pub fn crafting_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Inventory, &WantsToCraft)>,
    recipe_manager: Res<RecipeManagerResource>,
) {
    for (entity, mut inventory, wants_to_craft) in query.iter_mut() {
        let recipe_manager = &recipe_manager.0;
        match recipe_manager.get_required_resources(&wants_to_craft.item_name, 1) {
            Ok(required_resources) => {
                if inventory.remove_resources(&required_resources) {
                    inventory.add_item(&wants_to_craft.item_name, 1);
                }
            }
            Err(e) => {
                error!(
                    "Failed to get required resources for item '{}': {}",
                    wants_to_craft.item_name, e
                );
            }
        }
        commands.entity(entity).remove::<WantsToCraft>();
    }
}
