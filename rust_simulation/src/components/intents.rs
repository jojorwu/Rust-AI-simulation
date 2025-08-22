//! This module defines 'intent' components.
//! These are added to entities by the `goal_selection_system` to trigger
//! the appropriate action systems. They are typically removed by the action
//! system once the action is complete or fails.

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct IntendsToGather(pub String);

#[derive(Component)]
pub struct IntendsToCraft(pub String);

#[derive(Component)]
pub struct IntendsToBuild(pub String);

#[derive(Component)]
pub struct IntendsToAttack(pub Entity);

#[derive(Component)]
pub struct IntendsToFlee;

#[derive(Component)]
pub struct IntendsToExplore;

#[derive(Component)]
pub struct IntendsToStockpile(pub String);

#[derive(Component)]
pub struct WantsToEquip(pub String);

#[derive(Component)]
pub struct IntendsToEquip(pub String);
