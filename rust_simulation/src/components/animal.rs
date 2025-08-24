use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Pig;

#[derive(Component)]
pub struct Fleeing;

#[derive(Component)]
pub struct Hunger {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct SimpleAi {
    pub move_timer: u32,
    pub direction: (i32, i32),
}

impl Default for SimpleAi {
    fn default() -> Self {
        Self {
            move_timer: 0,
            direction: (0, 0),
        }
    }
}
