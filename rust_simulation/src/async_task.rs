use crate::components::ai::MentalMap;
use bevy_ecs::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use std::collections::VecDeque;

use crate::recipes::RecipeManager;
use std::collections::HashMap;
use std::sync::Arc;
use crate::components::Inventory;

// --- Task Data Payloads ---
// These structs are used to package up the data needed for a task.
// They are not sent over a channel, but moved directly into the async task closure.

pub struct PathfindingTask {
    pub entity: Entity,
    pub start: (u32, u32),
    pub goal: (u32, u32),
    pub mental_map: MentalMap,
}

pub struct CraftingTask {
    pub entity: Entity,
    pub item_name: String,
    pub inventory: Inventory,
    pub recipe_manager: Arc<RecipeManager>,
}


// --- Result Definitions ---

/// The result of an asynchronous task.
/// This is sent from a background thread to the main thread.
#[derive(Debug)]
pub enum AsyncResult {
    Pathfinding(PathfindingResult),
    Crafting(CraftingResult),
    // Other results like Crafting, Gathering, etc., will be added here.
}

/// The result of a pathfinding calculation.
#[derive(Debug)]
pub struct PathfindingResult {
    pub entity: Entity,
    pub path: Option<VecDeque<(u32, u32)>>,
}

/// The result of a crafting calculation.
#[derive(Debug)]
pub struct CraftingResult {
    pub entity: Entity,
    pub item_name: String,
    pub required_resources: HashMap<String, u32>,
    pub success: bool,
}


// --- Channel Resource ---

/// A Bevy resource that holds the MPSC channel for receiving asynchronous results.
#[derive(Resource)]
pub struct AsyncResultChannel {
    pub sender: Sender<AsyncResult>,
    pub receiver: Receiver<AsyncResult>,
}

impl Default for AsyncResultChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}
