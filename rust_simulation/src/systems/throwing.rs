use crate::{
    components::{
        intents::WantsToThrow, DroppedItem, Health, Inventory, Position,
    },
    ItemRegistryResource,
};
use bevy_ecs::prelude::*;

const THROW_RANGE: u32 = 5;
const MISS_CHANCE: f32 = 0.2;

pub fn throwing_system(
    mut commands: Commands,
    mut thrower_query: Query<(Entity, &mut Inventory, &Position, &WantsToThrow)>,
    mut target_query: Query<(Entity, &Position, &mut Health)>,
    mut dropped_item_query: Query<(&Position, &mut DroppedItem)>,
    item_registry: Res<ItemRegistryResource>,
) {
    for (thrower_entity, mut inventory, thrower_pos, wants_to_throw) in thrower_query.iter_mut() {
        // 1. Check if the thrower has the item and if it's throwable.
        if !inventory.has_item(&wants_to_throw.item_name, 1) {
            commands.entity(thrower_entity).remove::<WantsToThrow>();
            continue;
        }
        let item_def = item_registry.0.get_item(&wants_to_throw.item_name).unwrap();
        let throw_damage = if let Some(damage) = item_def.throw_damage {
            damage
        } else {
            // Not a throwable item.
            commands.entity(thrower_entity).remove::<WantsToThrow>();
            continue;
        };

        // 2. Consume the item from inventory.
        inventory.remove_item(&wants_to_throw.item_name, 1);

        // 3. Determine if the throw hits.
        let (target_entity, target_pos, mut target_health) =
            if let Ok(result) = target_query.get_mut(wants_to_throw.target) {
                result
            } else {
                // Target is gone.
                commands.entity(thrower_entity).remove::<WantsToThrow>();
                continue;
            };

        let distance = thrower_pos.x.abs_diff(target_pos.x) + thrower_pos.y.abs_diff(target_pos.y);
        let hit = distance <= THROW_RANGE && rand::random::<f32>() > MISS_CHANCE;

        if hit {
            // 4a. If it hits, apply damage.
            target_health.current -= throw_damage as i32;
        } else {
            // 4b. If it misses, drop the item on the target's tile.
            let mut item_stacked = false;
            for (dropped_pos, mut dropped_item) in dropped_item_query.iter_mut() {
                if dropped_pos == target_pos && dropped_item.item_name == wants_to_throw.item_name {
                    dropped_item.quantity += 1;
                    item_stacked = true;
                    break;
                }
            }
            if !item_stacked {
                commands.spawn((
                    *target_pos,
                    DroppedItem {
                        item_name: wants_to_throw.item_name.clone(),
                        quantity: 1,
                    },
                ));
            }
        }

        // 5. Remove the intent.
        commands.entity(thrower_entity).remove::<WantsToThrow>();
    }
}
