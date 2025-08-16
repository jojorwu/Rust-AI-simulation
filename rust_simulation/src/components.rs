use crate::ecs::{Component, Entity};

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

#[derive(Debug, Clone, Copy)]
pub struct WantsToGather {
    pub target: Entity,
}

impl Component for WantsToGather {}

#[derive(Debug, Clone, Copy)]
pub struct Resource {
    pub resource_type: char,
    pub quantity: u32,
}

impl Component for Resource {}
