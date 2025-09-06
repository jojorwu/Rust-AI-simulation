//! This module contains the core AI logic for the simulation agents.
//!
//! The main components are:
//! - `Goal`: An enum representing the high-level objectives an agent can have.
//! - `Brain`: A struct that encapsulates the decision-making logic for an agent.
//! - `BrainAction`: An enum representing the concrete actions an agent can take.
//!
//! The `Brain` uses a Q-learning-based approach to decide on a `Goal`, and then
//! uses a planner to break that goal down into a series of actions.

use crate::components::{Inventory, Velocity, WantsToAttack, WantsToCraft, WantsToStoreItem};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// A tile as remembered by the agent.
#[derive(Debug, Clone)]
pub struct MemoryTile {
    pub tile: super::map::Tile,
}

/// The relationship status between agents.
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipStatus {
    /// The other agent is considered hostile.
    Hostile,
}

/// A memory of another player.
#[derive(Debug, Clone)]
pub struct PlayerMemory {
    pub relationship: RelationshipStatus,
}

/// Represents the high-level goals that an agent can have.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Goal {
    /// Gather a specific resource.
    GatherResource(String),
    /// Craft a specific item.
    CraftItem(String),
    /// Build a specific structure.
    Build(String),
    /// Attack a specific entity.
    Attack(Entity),
    /// Flee from a threat.
    Flee,
    /// Explore the map to find resources.
    Explore,
    /// Stockpile a resource in a chest.
    Stockpile(String),
    /// Eat a food item.
    EatFood(String),
}

/// Represents the concrete actions that an agent's brain can decide to take.
#[derive(Debug)]
pub enum BrainAction {
    /// Move in a specific direction.
    Move(Velocity),
    /// Craft an item.
    Craft(WantsToCraft),
    /// Attack a target entity.
    Attack(WantsToAttack),
    /// Store an item in a chest.
    Store(WantsToStoreItem),
}

/// A summary of the agent's inventory, used as part of the `HighLevelState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventorySummary {
    pub items: HashMap<String, u32>,
}

impl Hash for InventorySummary {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut pairs: Vec<_> = self.items.iter().collect();
        pairs.sort_by_key(|(k, _)| *k);
        for (key, value) in pairs {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl From<&Inventory> for InventorySummary {
    fn from(inventory: &Inventory) -> Self {
        InventorySummary {
            items: inventory.items.clone(),
        }
    }
}

/// Represents the high-level state of the agent and its environment.
/// This is used as the input to the Q-learning model for goal selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HighLevelState {
    /// A summary of the agent's inventory.
    pub inventory_summary: InventorySummary,
    /// The number of hostile players the agent is aware of.
    pub num_hostile_players: u32,
    /// The agent's current health level.
    pub health_level: u32,
    /// The agent's current hunger level.
    pub hunger_level: u32,
    /// Whether it is currently night time.
    pub is_night: bool,
}
