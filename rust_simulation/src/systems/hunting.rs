use crate::{
    components::{
        intents::{IntendsToExplore, IntendsToGather, WantsToAttack},
        path::PathRequest,
        Position,
    },
    spatial::{SpatialIndex, SpatialPoint},
};
use bevy_ecs::prelude::*;

pub fn hunting_system(
    mut commands: Commands,
    spatial_index: Res<SpatialIndex>,
    hunter_query: Query<(Entity, &Position, &IntendsToGather), Without<PathRequest>>,
    position_query: Query<&Position>,
) {
    for (hunter_entity, hunter_pos, intends_to_gather) in hunter_query.iter() {
        if intends_to_gather.0 == "pig" {
            let query_point = SpatialPoint {
                x: hunter_pos.x as i32,
                y: hunter_pos.y as i32,
                entity: Entity::from_raw(0), // Dummy entity
            };
            if let Some(closest_animal) = spatial_index.animals.nearest_neighbor(&query_point)
            {
                let dist_x = (hunter_pos.x as i32 - closest_animal.x).abs();
                let dist_y = (hunter_pos.y as i32 - closest_animal.y).abs();
                let dist = dist_x + dist_y;

                if dist < 2 {
                    // Close enough to attack
                    commands
                        .entity(hunter_entity)
                        .insert(WantsToAttack {
                            target: closest_animal.entity,
                        })
                        .remove::<IntendsToGather>();
                } else {
                    // Pathfind to the animal
                    if let Ok(animal_pos) = position_query.get(closest_animal.entity) {
                        commands.entity(hunter_entity).insert(PathRequest {
                            start: (hunter_pos.x, hunter_pos.y),
                            goal: (animal_pos.x, animal_pos.y),
                        });
                    }
                }
            } else {
                // No animals found, so explore
                commands.entity(hunter_entity).insert(IntendsToExplore);
                commands.entity(hunter_entity).remove::<IntendsToGather>();
            }
        }
    }
}
