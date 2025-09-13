use crate::{
    components::{Inventory, WantsToCraft},
    events::Event,
    RecipeManagerResource,
};
use bevy_ecs::prelude::*;
use log::error;

pub fn crafting_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Inventory, &WantsToCraft)>,
    recipe_manager: Res<RecipeManagerResource>,
    mut event_writer: EventWriter<Event>,
) {
    for (entity, mut inventory, wants_to_craft) in query.iter_mut() {
        let recipe_manager = &recipe_manager.0;
        match recipe_manager.get_required_resources(&wants_to_craft.item_name, wants_to_craft.quantity) {
            Ok(required_resources) => {
                if inventory.has_resources(&required_resources) {
                    inventory.remove_resources(&required_resources);
                    inventory.add_item(&wants_to_craft.item_name, wants_to_craft.quantity);
                } else {
                    event_writer.send(Event::CraftingFailed {
                        crafter: entity,
                        item_name: wants_to_craft.item_name.clone(),
                    });
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
