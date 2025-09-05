use crate::components::intents::WantsToEat;
use crate::components::status::Hunger;
use crate::components::Inventory;
use crate::item::ItemRegistry;
use bevy_ecs::prelude::*;

pub fn eating_system(
    mut commands: Commands,
    mut query: Query<(Entity, &WantsToEat, &mut Inventory, &mut Hunger)>,
    item_registry: Res<ItemRegistry>,
) {
    for (entity, wants_to_eat, mut inventory, mut hunger) in query.iter_mut() {
        if let Some(item) = item_registry.get_item(&wants_to_eat.0) {
            if let Some(properties) = &item.properties {
                if let Some(hunger_value) = properties.get("hunger") {
                    if inventory.remove_item(&wants_to_eat.0, 1) {
                        hunger.current += *hunger_value as f32;
                        if hunger.current > hunger.max {
                            hunger.current = hunger.max;
                        }
                        commands.entity(entity).remove::<WantsToEat>();
                    }
                }
            }
        }
    }
}
