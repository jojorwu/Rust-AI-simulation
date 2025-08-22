use crate::{
    components::{ai::MentalMap, Inventory, Position},
    recipes::RecipeManager,
};
use bevy_ecs::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use std::{collections::{HashMap, VecDeque}, sync::Arc};

// --- Task Data Payloads ---

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

pub struct GatheringTask {
    pub agent_entity: Entity,
    pub resource_entity: Entity,
    pub resource_name: String,
    pub resource_quantity: u32,
}

pub struct BuildingTask {
    pub builder_entity: Entity,
    pub position: Position,
    pub inventory: Inventory,
    pub structure_name: String,
    pub recipe_manager: Arc<RecipeManager>,
}

// --- Result Definitions ---

#[derive(Debug)]
pub enum AsyncResult {
    Pathfinding(PathfindingResult),
    Crafting(CraftingResult),
    Gathering(GatheringResult),
    Building(BuildingResult),
}

#[derive(Debug)]
pub struct PathfindingResult {
    pub entity: Entity,
    pub path: Option<VecDeque<(u32, u32)>>,
}

#[derive(Debug)]
pub struct CraftingResult {
    pub entity: Entity,
    pub item_name: String,
    pub required_resources: HashMap<String, u32>,
    pub success: bool,
}

#[derive(Debug)]
pub struct GatheringResult {
    pub agent_entity: Entity,
    pub resource_entity: Entity,
    pub resource_name: String,
    pub gathered_amount: u32,
    pub despawn_resource: bool,
}

#[derive(Debug)]
pub struct BuildingResult {
    pub builder_entity: Entity,
    pub position: Position,
    pub structure_name: String,
    pub required_resources: HashMap<String, u32>,
    pub success: bool,
}

// --- Channel Resource ---

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
