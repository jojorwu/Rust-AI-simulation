use crate::animals::pig::Pig;
use crate::components::intents::{IntendsToExplore, IntendsToGather};
use crate::components::path::PathRequest;
use crate::components::{intents::WantsToAttack, Position};
use bevy_ecs::prelude::*;

pub fn hunting_system(
    mut commands: Commands,
    hunter_query: Query<(Entity, &Position, &IntendsToGather), Without<PathRequest>>,
    pig_query: Query<(Entity, &Position), With<Pig>>,
) {
    for (hunter_entity, hunter_pos, intends_to_gather) in hunter_query.iter() {
        if intends_to_gather.0 == "pig" {
            if pig_query.is_empty() {
                // No pigs found, so explore
                commands.entity(hunter_entity).insert(IntendsToExplore);
                commands.entity(hunter_entity).remove::<IntendsToGather>();
                continue;
            }

            let mut closest_pig: Option<(Entity, f32)> = None;
            for (pig_entity, pig_pos) in pig_query.iter() {
                let dist_x = (hunter_pos.x as i32 - pig_pos.x as i32).abs();
                let dist_y = (hunter_pos.y as i32 - pig_pos.y as i32).abs();
                let dist = (dist_x + dist_y) as f32; // Manhattan distance

                let is_closer = closest_pig.is_none_or(|(_, current_dist)| dist < current_dist);
                if is_closer {
                    closest_pig = Some((pig_entity, dist));
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
                    if let Ok((_, pig_pos)) = pig_query.get(pig_entity) {
                        commands.entity(hunter_entity).insert(PathRequest {
                            start: (hunter_pos.x, hunter_pos.y),
                            goal: (pig_pos.x, pig_pos.y),
                        });
                    }
                }
            }
        }
    }
}
