use crate::ecs::Component;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Component for Position {}

#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub dx: i32,
    pub dy: i32,
}

impl Component for Velocity {}
