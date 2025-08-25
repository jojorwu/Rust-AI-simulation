use bevy::prelude::*;
use crate::animals::pig::Pig;
use crate::components::{Position, Velocity, WantsToAttack};
use crate::components::status::Hunger;
use crate::player::Player;
use rand::Rng;
use crate::animals::pack::Pack;

#[derive(Component)]
pub struct Wolf;

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
pub enum WolfState {
    Wandering,
    Hunting,
    Following,
}

#[derive(Component)]
pub struct WolfAI {
    pub state: WolfState,
    pub target: Option<Entity>,
    pub timer: u32,
    pub pack: Option<Entity>,
}

impl Default for WolfAI {
    fn default() -> Self {
        Self {
            state: WolfState::Wandering,
            target: None,
            timer: 0,
            pack: None,
        }
    }
}

const WOLF_WANDER_TIMER: u32 = 60;
const WOLF_VIEW_DISTANCE: f32 = 10.0;
const WOLF_HUNT_THRESHOLD: f32 = 50.0;

pub fn wolf_ai_system(
    mut commands: Commands,
    mut wolf_query: Query<(Entity, &mut WolfAI, &mut Velocity, &Position, &Hunger), With<Wolf>>,
    prey_query: Query<(Entity, &Position), Or<(With<Player>, With<Pig>)>>,
    player_query: Query<(Entity, &Position), With<Player>>,
    pack_query: Query<&Pack>,
    position_query: Query<&Position>,
) {
    let mut rng = rand::thread_rng();
    for (wolf_entity, mut wolf_ai, mut velocity, position, hunger) in wolf_query.iter_mut() {
        if let Some(pack_entity) = wolf_ai.pack {
            if let Ok(pack) = pack_query.get(pack_entity) {
                if let Some(leader_entity) = pack.leader {
                    if wolf_entity == leader_entity {
                        // This wolf is the leader of the pack
                        if hunger.current >= WOLF_HUNT_THRESHOLD {
                            let mut closest_prey: Option<(Entity, f32)> = None;

                            // If pack size is 3 or more, prioritize players
                            if pack.members.len() >= 3 {
                                for (player_entity, player_position) in player_query.iter() {
                                    let distance = position.distance(player_position);
                                    if distance < WOLF_VIEW_DISTANCE {
                                        if let Some((_, min_dist)) = closest_prey {
                                            if distance < min_dist {
                                                closest_prey = Some((player_entity, distance));
                                            }
                                        } else {
                                            closest_prey = Some((player_entity, distance));
                                        }
                                    }
                                }
                            }

                            // If no player is found or pack size is less than 3, hunt any prey
                            if closest_prey.is_none() {
                                for (prey_entity, prey_position) in prey_query.iter() {
                                    let distance = position.distance(prey_position);
                                    if distance < WOLF_VIEW_DISTANCE {
                                        if let Some((_, min_dist)) = closest_prey {
                                            if distance < min_dist {
                                                closest_prey = Some((prey_entity, distance));
                                            }
                                        } else {
                                            closest_prey = Some((prey_entity, distance));
                                        }
                                    }
                                }
                            }


                            if let Some((target_entity, _)) = closest_prey {
                                wolf_ai.state = WolfState::Hunting;
                                wolf_ai.target = Some(target_entity);
                            }
                        } else {
                            wolf_ai.state = WolfState::Wandering;
                        }

                        if wolf_ai.state == WolfState::Hunting {
                            if let Some(target_entity) = wolf_ai.target {
                                if let Ok((_, target_position)) = prey_query.get(target_entity) {
                                    let dx = (target_position.x as i32 - position.x as i32).signum();
                                    let dy = (target_position.y as i32 - position.y as i32).signum();
                                    velocity.dx = dx;
                                    velocity.dy = dy;

                                    if position.distance(target_position) < 1.5 {
                                        commands.entity(wolf_entity).insert(WantsToAttack { target: target_entity });
                                    }
                                } else {
                                    wolf_ai.state = WolfState::Wandering;
                                    wolf_ai.target = None;
                                    wolf_ai.timer = 0;
                                }
                            } else {
                                wolf_ai.state = WolfState::Wandering;
                                wolf_ai.timer = 0;
                            }
                        } else { // Wandering
                            if wolf_ai.timer == 0 {
                                velocity.dx = rng.gen_range(-1..=1);
                                velocity.dy = rng.gen_range(-1..=1);
                                wolf_ai.timer = WOLF_WANDER_TIMER;
                            } else {
                                wolf_ai.timer -= 1;
                            }
                        }
                    } else {
                        // This wolf is a follower
                        if let Ok(leader_pos) = position_query.get(leader_entity) {
                            let distance = position.distance(leader_pos);
                            if distance > 2.0 {
                                let dx = (leader_pos.x as i32 - position.x as i32).signum();
                                let dy = (leader_pos.y as i32 - position.y as i32).signum();
                                velocity.dx = dx;
                                velocity.dy = dy;
                            } else {
                                velocity.dx = 0;
                                velocity.dy = 0;
                            }
                        } else {
                            // Leader is dead, dissolve the pack
                            wolf_ai.pack = None;
                        }
                    }
                } else {
                    // No leader, dissolve the pack
                    wolf_ai.pack = None;
                }
            } else {
                // Pack entity doesn't exist, dissolve the pack
                wolf_ai.pack = None;
            }
        } else {
            // Lone wolf behavior
            if hunger.current < WOLF_HUNT_THRESHOLD {
                wolf_ai.state = WolfState::Wandering;
            }

            if wolf_ai.state == WolfState::Wandering {
                if wolf_ai.timer == 0 {
                    velocity.dx = rng.gen_range(-1..=1);
                    velocity.dy = rng.gen_range(-1..=1);
                    wolf_ai.timer = WOLF_WANDER_TIMER;
                } else {
                    wolf_ai.timer -= 1;
                }
            }

            if hunger.current >= WOLF_HUNT_THRESHOLD {
                let mut closest_prey: Option<(Entity, f32)> = None;
                for (prey_entity, prey_position) in prey_query.iter() {
                    let distance = position.distance(prey_position);
                    if distance < WOLF_VIEW_DISTANCE {
                        if let Some((_, min_dist)) = closest_prey {
                            if distance < min_dist {
                                closest_prey = Some((prey_entity, distance));
                            }
                        } else {
                            closest_prey = Some((prey_entity, distance));
                        }
                    }
                }

                if let Some((target_entity, _)) = closest_prey {
                    wolf_ai.state = WolfState::Hunting;
                    wolf_ai.target = Some(target_entity);
                }
            }

            if wolf_ai.state == WolfState::Hunting {
                if let Some(target_entity) = wolf_ai.target {
                    if prey_query.get(target_entity).is_ok() {
                        if let Ok((_, target_position)) = prey_query.get(target_entity) {
                            let dx = (target_position.x as i32 - position.x as i32).signum();
                            let dy = (target_position.y as i32 - position.y as i32).signum();
                            velocity.dx = dx;
                            velocity.dy = dy;

                            if position.distance(target_position) < 1.5 {
                                commands.entity(wolf_entity).insert(WantsToAttack { target: target_entity });
                            }
                        }
                    } else {
                        wolf_ai.state = WolfState::Wandering;
                        wolf_ai.target = None;
                        wolf_ai.timer = 0;
                    }
                } else {
                    wolf_ai.state = WolfState::Wandering;
                    wolf_ai.timer = 0;
                }
            }
        }
    }
}
