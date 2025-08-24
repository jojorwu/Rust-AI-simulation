use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component)]
pub struct Hunger {
    pub current: f32,
    pub max: f32,
}
