use crate::ecs::{Component, Entity};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, Eq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Hash for Position {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
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

#[derive(Debug, Clone)]
pub struct WantsToCraft {
    pub item_name: String,
}

impl Component for WantsToCraft {}

#[derive(Debug, Clone)]
pub struct WantsToBuild {
    pub structure_name: String,
}

impl Component for WantsToBuild {}

#[derive(Debug, Clone, Copy)]
pub struct WantsToAttack {
    pub target: Entity,
}

impl Component for WantsToAttack {}

#[derive(Debug, Clone, Copy)]
pub struct WantsToPickup {}

impl Component for WantsToPickup {}

#[derive(Debug, Clone)]
pub struct Resource {
    pub name: String,
    pub quantity: u32,
}

impl Component for Resource {}

#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Component for Health {}

#[derive(Debug, Clone)]
pub struct DroppedItem {
    pub item_name: String,
    pub quantity: u32,
}

impl Component for DroppedItem {}
