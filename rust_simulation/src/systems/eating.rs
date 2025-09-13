use crate::components::intents::WantsToEat;
use crate::components::status::Hunger;
use crate::components::Inventory;
use crate::ItemRegistryResource;
use bevy::ecs::system::Commands;
use bevy_ecs::prelude::*;

pub fn eating_system(
    mut commands: Commands,
    mut query: Query<(Entity, &WantsToEat, &mut Inventory, &mut Hunger)>,
    item_registry: Res<ItemRegistryResource>,
) {
    for (entity, wants_to_eat, mut inventory, mut hunger) in query.iter_mut() {
        if let Some(item_def) = item_registry.0.get_item(&wants_to_eat.0) {
            if item_def.is_food && inventory.remove_item(&wants_to_eat.0, 1) {
                hunger.current += item_def.food_value;
                if hunger.current > hunger.max {
                    hunger.current = hunger.max;
                }
            }
        }
        commands.entity(entity).remove::<WantsToEat>();
    }
}
