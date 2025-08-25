use bevy::prelude::*;
use crate::animals::pig::Pig;
use crate::components::{Position, Velocity, WantsToAttack};
use crate::components::status::Hunger;
use crate::player::Player;
use rand::Rng;

#[derive(Component)]
pub struct Wolf;

#[derive(Component, PartialEq, Eq)]
pub enum WolfState {
    Wandering,
    Hunting,
}

#[derive(Component)]
pub struct WolfAI {
    pub state: WolfState,
    pub target: Option<Entity>,
    pub timer: u32,
}

impl Default for WolfAI {
    fn default() -> Self {
        Self {
            state: WolfState::Wandering,
            target: None,
            timer: 0,
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
) {
    let mut rng = rand::thread_rng();
    for (wolf_entity, mut wolf_ai, mut velocity, position, hunger) in wolf_query.iter_mut() {
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
        }
    }
}
