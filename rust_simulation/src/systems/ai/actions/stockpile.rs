use crate::components::{
    BrainComponent, Chest, Position, WantsToStoreItem,
    intents::IntendsToStockpile,
    path::{CurrentPath, PathRequest},
};
use bevy_ecs::prelude::*;

pub fn stockpile_action_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut BrainComponent,
            &Position,
            &IntendsToStockpile,
        ),
        (Without<CurrentPath>, Without<PathRequest>),
    >,
    chest_query: Query<(Entity, &Position, &Chest)>,
) {
    for (entity, mut brain, position, intent) in query.iter_mut() {
        let resource_name = &intent.0;

        let Some(home_base_pos) = brain.home_base else {
            // No home base, goal is impossible.
            commands.entity(entity).remove::<IntendsToStockpile>();
            brain.current_goal = None;
            continue;
        };

        if let Some((chest_entity, chest_pos)) = find_closest_chest(&chest_query, &home_base_pos) {
            let is_adjacent =
                (position.x.abs_diff(chest_pos.x) <= 1) && (position.y.abs_diff(chest_pos.y) <= 1);

            if is_adjacent {
                // If adjacent, add WantsToStoreItem.
                commands.entity(entity).insert(WantsToStoreItem {
                    item_name: resource_name.clone(),
                    quantity: 1, // Simplified
                    target_chest: chest_entity,
                });
                commands.entity(entity).remove::<IntendsToStockpile>();
                brain.current_goal = None;
            } else {
                // If not adjacent, request a path.
                commands.entity(entity).insert(PathRequest {
                    start: (position.x, position.y),
                    goal: (chest_pos.x, chest_pos.y),
                });
            }
        } else {
            // No chest found, goal is impossible.
            commands.entity(entity).remove::<IntendsToStockpile>();
            brain.current_goal = None;
        }
    }
}

fn find_closest_chest(
    chest_query: &Query<(Entity, &Position, &Chest)>,
    pos: &Position,
) -> Option<(Entity, Position)> {
    chest_query
        .iter()
        .map(|(e, p, _c)| (e, *p))
        .min_by_key(|(_, chest_pos)| chest_pos.x.abs_diff(pos.x) + chest_pos.y.abs_diff(pos.y))
}
