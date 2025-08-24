use crate::components::intents::WantsToEat;
use crate::components::status::Hunger;
use crate::components::Inventory;
use bevy_ecs::prelude::*;

const MEAT_HUNGER_VALUE: f32 = 25.0;

pub fn eating_system(
    mut commands: Commands,
    mut query: Query<(Entity, &WantsToEat, &mut Inventory, &mut Hunger)>,
) {
    for (entity, wants_to_eat, mut inventory, mut hunger) in query.iter_mut() {
        if inventory.remove_item(&wants_to_eat.0, 1) {
            hunger.current += MEAT_HUNGER_VALUE;
            if hunger.current > hunger.max {
                hunger.current = hunger.max;
            }
            commands.entity(entity).remove::<WantsToEat>();
        }
    }
}
