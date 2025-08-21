use crate::components::{Inventory, WantsToCraft};
use crate::lib::RecipeManagerResource;
use bevy_ecs::prelude::*;

pub fn crafting_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Inventory, &WantsToCraft)>,
    recipe_manager: Res<RecipeManagerResource>,
) {
    for (entity, mut inventory, wants_to_craft) in query.iter_mut() {
        let recipe_manager = &recipe_manager.0;
        let required_resources =
            recipe_manager.get_required_resources(&wants_to_craft.item_name, 1);
        if inventory.has_resources(&required_resources)
            && inventory.remove_resources(&required_resources)
        {
            inventory.add_item(&wants_to_craft.item_name, 1);
        }
        commands.entity(entity).remove::<WantsToCraft>();
    }
}
