use crate::{
    components::{
        intents::WantsToThrow, DroppedItem, Health, Inventory, Position,
    },
    config, ItemRegistryResource,
};
use bevy_ecs::prelude::*;

/// The core logic for resolving a thrown item. This is a public function so it
/// can be called directly from tests for deterministic outcomes.
pub fn resolve_throw(
    commands: &mut Commands,
    _thrower_entity: Entity,
    thrower_pos: &Position,
    item_name: &str,
    _target_entity: Entity,
    target_pos: &Position,
    target_health: &mut Health,
    hit_chance: f32,
    damage: u32,
    dropped_item_query: &mut Query<(&Position, &mut DroppedItem)>,
) {
    let distance = thrower_pos.x.abs_diff(target_pos.x) + thrower_pos.y.abs_diff(target_pos.y);
    let hit = distance <= config::THROW_RANGE && hit_chance > config::MISS_CHANCE;

    if hit {
        // Apply damage.
        target_health.current -= damage as i32;
    } else {
        // Drop the item on the target's tile.
        let mut item_stacked = false;
        for (dropped_pos, mut dropped_item) in dropped_item_query.iter_mut() {
            if dropped_pos == target_pos && dropped_item.item_name == item_name {
                dropped_item.quantity += 1;
                item_stacked = true;
                break;
            }
        }
        if !item_stacked {
            commands.spawn((
                *target_pos,
                DroppedItem {
                    item_name: item_name.to_string(),
                    quantity: 1,
                },
            ));
        }
    }
}

pub fn throwing_system(
    mut commands: Commands,
    mut thrower_query: Query<(Entity, &mut Inventory, &Position, &WantsToThrow)>,
    mut target_query: Query<(Entity, &Position, &mut Health)>,
    mut dropped_item_query: Query<(&Position, &mut DroppedItem)>,
    item_registry: Res<ItemRegistryResource>,
) {
    for (thrower_entity, mut inventory, thrower_pos, wants_to_throw) in thrower_query.iter_mut() {
        let damage = if let Some(item_def) = item_registry.0.get_item(&wants_to_throw.item_name) {
            if let Some(damage) = item_def.throw_damage {
                damage
            } else {
                commands.entity(thrower_entity).remove::<WantsToThrow>();
                continue;
            }
        } else {
            commands.entity(thrower_entity).remove::<WantsToThrow>();
            continue;
        };

        if !inventory.remove_item(&wants_to_throw.item_name, 1) {
            // Should not happen if we add a has_item check, but as a safeguard.
            commands.entity(thrower_entity).remove::<WantsToThrow>();
            continue;
        }

        if let Ok((target_entity, target_pos, mut target_health)) = target_query.get_mut(wants_to_throw.target) {
            resolve_throw(
                &mut commands,
                thrower_entity,
                thrower_pos,
                &wants_to_throw.item_name,
                target_entity,
                target_pos,
                &mut target_health,
                rand::random::<f32>(),
                damage,
                &mut dropped_item_query,
            );
        }

        commands.entity(thrower_entity).remove::<WantsToThrow>();
    }
}
