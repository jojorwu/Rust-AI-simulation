use crate::components::intents::WantsToEat;
use crate::components::status::Hunger;
use crate::components::Inventory;
use crate::ItemRegistryResource;
use bevy::ecs::system::ParallelCommands;
use bevy_ecs::prelude::*;
use rayon::prelude::*;

pub fn eating_system(
    commands: ParallelCommands,
    mut query: Query<(Entity, &WantsToEat, &mut Inventory, &mut Hunger)>,
    item_registry: Res<ItemRegistryResource>,
) {
    query
        .par_iter_mut()
        .for_each(|(entity, wants_to_eat, mut inventory, mut hunger)| {
            if let Some(item_def) = item_registry.0.get_item(&wants_to_eat.0) {
                if item_def.is_food && inventory.remove_item(&wants_to_eat.0, 1) {
                    hunger.current += item_def.food_value;
                    if hunger.current > hunger.max {
                        hunger.current = hunger.max;
                    }
                    commands.command_scope(|mut c| {
                        c.entity(entity).remove::<WantsToEat>();
                    });
                }
            }
        });
}
