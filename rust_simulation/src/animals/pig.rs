use crate::components::{Position, Velocity, WantsToAttack};
use bevy_ecs::prelude::*;
use rand::Rng;

// --- Components ---

#[derive(Component)]
pub struct Pig;

#[derive(Component)]
pub struct Fleeing;

#[derive(Component, Default)]
pub struct SimpleAi {
    pub move_timer: u32,
    pub direction: (i32, i32),
}

// --- Systems ---

type WanderingPigQuery<'w, 's> =
    Query<'w, 's, (&'s mut SimpleAi, &'s mut Velocity), (With<Pig>, Without<Fleeing>)>;

const WANDER_TIMER: u32 = 60; // Change direction every 60 ticks
const FLEE_TIMER: u32 = 120; // Flee for 120 ticks

pub fn wandering_system(mut query: WanderingPigQuery) {
    let mut rng = rand::rng();
    for (mut ai, mut velocity) in query.iter_mut() {
        if ai.move_timer == 0 {
            let dx = rng.random_range(-1..=1);
            let dy = rng.random_range(-1..=1);
            ai.direction = (dx, dy);
            velocity.dx = dx;
            velocity.dy = dy;
            ai.move_timer = WANDER_TIMER;
        } else {
            ai.move_timer -= 1;
        }
    }
}

pub fn fleeing_system(
    mut commands: Commands,
    attack_query: Query<(Entity, &WantsToAttack)>,
    mut pig_query: Query<(Entity, &mut SimpleAi, &mut Velocity), With<Pig>>,
    position_query: Query<&Position>,
) {
    for (attacker_entity, attack) in attack_query.iter() {
        if let Ok((pig_entity, mut ai, mut velocity)) = pig_query.get_mut(attack.target) {
            if let Ok(attacker_pos) = position_query.get(attacker_entity) {
                if let Ok(pig_pos) = position_query.get(pig_entity) {
                    let dx = pig_pos.x as i32 - attacker_pos.x as i32;
                    let dy = pig_pos.y as i32 - attacker_pos.y as i32;

                    let (flee_dx, flee_dy) = if dx == 0 && dy == 0 {
                        // If positions are the same, flee in a random direction
                        let mut rng = rand::rng();
                        let mut flee_dx = 0;
                        let mut flee_dy = 0;
                        while flee_dx == 0 && flee_dy == 0 {
                            flee_dx = rng.random_range(-1..=1);
                            flee_dy = rng.random_range(-1..=1);
                        }
                        (flee_dx, flee_dy)
                    } else {
                        (dx.signum(), dy.signum())
                    };

                    ai.direction = (flee_dx, flee_dy);
                    velocity.dx = flee_dx;
                    velocity.dy = flee_dy;
                    ai.move_timer = FLEE_TIMER;
                    commands.entity(pig_entity).insert(Fleeing);
                }
            }
        }
    }
}

pub fn flee_stop_system(mut commands: Commands, query: Query<(Entity, &SimpleAi), With<Fleeing>>) {
    for (entity, ai) in query.iter() {
        if ai.move_timer == 0 {
            commands.entity(entity).remove::<Fleeing>();
        }
    }
}
