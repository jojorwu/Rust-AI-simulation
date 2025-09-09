//! This module defines components related to the status of an entity, such as health and hunger.

use bevy_ecs::prelude::*;

/// A component representing the health of an entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    /// The current health value.
    pub current: i32,
    /// The maximum possible health value.
    pub max: i32,
}

/// A component representing the hunger of an entity.
///
/// Hunger decreases over time and can be restored by eating.
#[derive(Component)]
pub struct Hunger {
    /// The current hunger value.
    pub current: f32,
    /// The maximum possible hunger value.
    pub max: f32,
}

/// A component representing an instance of damage to be applied to an entity.
///
/// This is a 'command' component, typically added and removed in the same frame.
#[derive(Component)]
pub struct Damage(pub i32);
