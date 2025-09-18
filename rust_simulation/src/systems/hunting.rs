use crate::animals::pig::Pig;
use crate::components::intents::IntendsToGather;
use crate::components::path::PathRequest;
use crate::components::{intents::WantsToAttack, Position};
use crate::map::Map;
use bevy_ecs::prelude::*;

pub fn hunting_system(
    mut commands: Commands,
    hunter_query: Query<(Entity, &Position, &IntendsToGather), Without<PathRequest>>,
    pig_query: Query<Entity, With<Pig>>, // Query for pig entities, not their positions
    position_query: Query<&Position>,    // A general query for positions
    map: Res<Map>,
) {
    for (hunter_entity, hunter_pos, intends_to_gather) in hunter_query.iter() {
        if intends_to_gather.0 == "pig" {
            let mut closest_pig: Option<(Entity, f32)> = None;

            // Search in a 5x5 chunk area around the hunter
            if let Some((chunk_x, chunk_y)) = map.get_chunk_index(hunter_pos.x, hunter_pos.y) {
                for x_offset in -2..=2 {
                    for y_offset in -2..=2 {
                        let check_chunk_x = chunk_x as i32 + x_offset;
                        let check_chunk_y = chunk_y as i32 + y_offset;

                        if check_chunk_x >= 0 && check_chunk_y >= 0 {
                            if let Some(entities) = map.get_entities_in_chunk(check_chunk_x as u32, check_chunk_y as u32) {
                                for &entity in &entities {
                                    if pig_query.get(entity).is_ok() {
                                        if let Ok(pig_pos) = position_query.get(entity) {
                                            let dist = hunter_pos.distance(pig_pos);
                                            if closest_pig.is_none() || dist < closest_pig.unwrap().1 {
                                                closest_pig = Some((entity, dist));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some((pig_entity, dist)) = closest_pig {
                if dist < 2.0 {
                    // Close enough to attack
                    commands
                        .entity(hunter_entity)
                        .insert(WantsToAttack { target: pig_entity });
                    commands.entity(hunter_entity).remove::<IntendsToGather>();
                } else {
                    // Pathfind to the pig
                    if let Ok(pig_pos) = position_query.get(pig_entity) {
                        commands.entity(hunter_entity).insert(PathRequest {
                            start: (hunter_pos.x, hunter_pos.y),
                            goal: (pig_pos.x, pig_pos.y),
                        });
                        commands.entity(hunter_entity).remove::<IntendsToGather>();
                    }
                }
            }
        }
    }
}
