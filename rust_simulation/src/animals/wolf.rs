use bevy::prelude::*;
use crate::animals::pig::Pig;
use crate::components::{Position, Velocity, WantsToAttack};
use crate::map::Map;
use rand::Rng;

#[derive(Component)]
pub struct Wolf;

#[derive(Component)]
pub enum WolfState {
    Wandering,
    Hunting(Entity),
}

#[derive(Component)]
pub struct WolfAi {
    pub state: WolfState,
    pub state_timer: u32,
}

impl Default for WolfAi {
    fn default() -> Self {
        Self {
            state: WolfState::Wandering,
            state_timer: 0,
        }
    }
}

const WOLF_WANDER_TIMER: u32 = 60;
const WOLF_VIEW_DISTANCE: f32 = 10.0;

pub fn wolf_ai_system(
    mut commands: Commands,
    mut wolf_query: Query<(Entity, &mut WolfAi, &Position, &mut Velocity), With<Wolf>>,
    pig_query: Query<(Entity, &Position), With<Pig>>,
    map: Res<Map>,
) {
    let mut rng = rand::thread_rng();

    for (wolf_entity, mut wolf_ai, wolf_pos, mut velocity) in wolf_query.iter_mut() {
        match wolf_ai.state {
            WolfState::Wandering => {
                if wolf_ai.state_timer == 0 {
                    let dx = rng.gen_range(-1..=1);
                    let dy = rng.gen_range(-1..=1);
                    velocity.dx = dx;
                    velocity.dy = dy;
                    wolf_ai.state_timer = WOLF_WANDER_TIMER;
                } else {
                    wolf_ai.state_timer -= 1;
                }

                // Check for nearby pigs
                for (pig_entity, pig_pos) in pig_query.iter() {
                    if wolf_pos.distance(pig_pos) < WOLF_VIEW_DISTANCE {
                        wolf_ai.state = WolfState::Hunting(pig_entity);
                        break;
                    }
                }
            }
            WolfState::Hunting(target) => {
                if let Ok((_target_entity, target_pos)) = pig_query.get(target) {
                    let dx = (target_pos.x as i32 - wolf_pos.x as i32).signum();
                    let dy = (target_pos.y as i32 - wolf_pos.y as i32).signum();
                    velocity.dx = dx;
                    velocity.dy = dy;

                    if wolf_pos.distance(target_pos) < 1.5 {
                        commands.entity(wolf_entity).insert(WantsToAttack { target });
                    }
                } else {
                    // The target is gone, go back to wandering
                    wolf_ai.state = WolfState::Wandering;
                    wolf_ai.state_timer = 0;
                }
            }
        }
    }
}
