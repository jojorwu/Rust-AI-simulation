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
use std::collections::{BTreeMap, HashMap};
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
///
/// This enum is used by the Q-learning model to represent the possible objectives
/// an agent can choose to pursue.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Goal {
    /// A goal to gather a specific resource (e.g., "wood", "stone").
    GatherResource(String),
    /// A goal to craft a specific item from a recipe (e.g., "stone_axe").
    CraftItem(String),
    /// A goal to build a specific structure (e.g., "chest").
    Build(String),
    /// A goal to attack another entity.
    Attack(Entity),
    /// A goal to flee from a perceived threat.
    Flee,
    /// A goal to explore the map to discover new resources.
    Explore,
    /// A goal to stockpile a resource in a storage container.
    Stockpile(String),
    /// A goal to eat a food item to restore hunger.
    EatFood(String),
}

/// Represents the concrete actions that an agent's brain can decide to take.
///
/// These actions are the output of the AI's planning process and are translated
/// into commands that modify the agent's state or the game world.
#[derive(Debug)]
pub enum BrainAction {
    /// An action to move the agent in a specific direction.
    Move(Velocity),
    /// An action to craft a specific item.
    Craft(WantsToCraft),
    /// An action to attack a target entity.
    Attack(WantsToAttack),
    /// An action to store an item in a storage container.
    Store(WantsToStoreItem),
}

/// A summary of the agent's inventory, used as part of the `HighLevelState`.
///
/// This summary is a simplified representation of the agent's inventory,
/// mapping item names to their quantities. It uses a `BTreeMap` to ensure
/// that the hash is deterministic, which is crucial for the Q-learning state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct InventorySummary {
    /// A map of item names to their counts.
    pub items: BTreeMap<String, u32>,
}

impl From<&Inventory> for InventorySummary {
    fn from(inventory: &Inventory) -> Self {
        // Collect into a BTreeMap to ensure deterministic ordering for hashing.
        let items = inventory.items.clone().into_iter().collect();
        InventorySummary { items }
    }
}

/// Represents a discretized level for a continuous statistic like health or hunger.
/// This is used to simplify the state space for the Q-learning algorithm.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DiscretizedLevel {
    /// Represents a low level of the statistic (e.g., 0-33%).
    Low,
    /// Represents a medium level of the statistic (e.g., 34-66%).
    Medium,
    /// Represents a high level of the statistic (e.g., 67-100%).
    High,
}

/// Represents the high-level, discretized state of an agent and its environment.
///
/// This struct is used as the key in the Q-table for the Q-learning algorithm.
/// All its fields must be discrete and hashable to define a finite state space.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HighLevelState {
    /// A summary of the agent's current inventory.
    pub inventory_summary: InventorySummary,
    /// The number of hostile players the agent is aware of.
    pub num_hostile_players: u32,
    /// The agent's discretized health level.
    pub health_level: DiscretizedLevel,
    /// The agent's discretized hunger level.
    pub hunger_level: DiscretizedLevel,
    /// Whether it is currently night time in the simulation.
    pub is_night: bool,
}
