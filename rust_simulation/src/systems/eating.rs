use crate::components::intents::WantsToEat;
use crate::components::status::Hunger;
use crate::components::Inventory;
use crate::config::Config;
use bevy_ecs::prelude::*;

pub fn eating_system(
    mut commands: Commands,
    mut query: Query<(Entity, &WantsToEat, &mut Inventory, &mut Hunger)>,
    config: Res<Config>,
) {
    for (entity, wants_to_eat, mut inventory, mut hunger) in query.iter_mut() {
        if inventory.remove_item(&wants_to_eat.0, 1) {
            hunger.current += config.survival.meat_hunger_value;
            if hunger.current > hunger.max {
                hunger.current = hunger.max;
            }
            commands.entity(entity).remove::<WantsToEat>();
        }
    }
}
