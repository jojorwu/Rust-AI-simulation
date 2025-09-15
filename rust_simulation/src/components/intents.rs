//! This module defines 'intent' components.
//!
//! These are 'marker' or 'flag' components that represent an agent's current intent.
//! They are typically added to an entity by a planning system (like `goal_selection_system`)
//! to signal that a specific action should be performed. The corresponding action system
//! then queries for entities with these components, performs the action, and removes
//! the intent component.

use bevy_ecs::prelude::*;

/// An intent to gather a resource of the specified type to a target amount.
#[derive(Component)]
pub struct IntendsToGather(pub String, pub u32);

/// An intent to gather a resource from a specific target entity (e.g., a tree).
#[derive(Component)]
pub struct IntendsToGatherFrom(pub Entity);

/// A component indicating that an agent is in the process of gathering a resource.
#[derive(Component)]
pub struct IsGathering {
    /// The specific entity being gathered from.
    pub target: Entity,
    /// The name of the resource being gathered.
    pub resource: String,
    /// The target amount to gather.
    pub amount: u32,
}

/// An intent to craft an item with the specified name.
#[derive(Component)]
pub struct IntendsToCraft(pub String);

/// An intent to build a structure with the specified name.
#[derive(Component)]
pub struct IntendsToBuild(pub String);

/// An intent to attack a specific target entity.
#[derive(Component)]
pub struct IntendsToAttack(pub Entity);

/// An intent to flee from a threat.
#[derive(Component)]
pub struct IntendsToFlee;

/// An intent to explore the map.
#[derive(Component)]
pub struct IntendsToExplore;

/// An intent to stockpile a resource of the specified type.
#[derive(Component)]
pub struct IntendsToStockpile(pub String);

/// An intent to eat a food item with the specified name.
#[derive(Component)]
pub struct WantsToEat(pub String);

/// An intermediate intent to check if the required resources for a recipe are available.
#[derive(Component)]
pub struct CheckResources(pub String);

/// A marker component indicating that the agent has the necessary resources for a task.
#[derive(Component)]
pub struct HasResources;

/// An intermediate intent to check if a tile at a position is suitable for building.
#[derive(Component)]
pub struct CheckTile(pub super::Position);

/// A marker component indicating that a tile is suitable for a task.
#[derive(Component)]
pub struct TileIsSuitable;

#[derive(Component, Debug, Clone, Copy)]
pub struct WantsToGather {
    pub target: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToCraft {
    pub item_name: String,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToBuild {
    pub structure_name: String,
}

#[derive(Component, Clone, Debug)]
pub struct WantsToStoreItem {
    pub item_name: String,
    pub quantity: u32,
    pub target_chest: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WantsToAttack {
    pub target: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WantsToPickup {}
