use crate::ecs::World;
use crate::events::EventBus;
use crate::item::ItemRegistry;
use crate::map::Map;
use crate::recipes::RecipeManager;
use std::sync::{Arc, Mutex};

/// A container for the game state that can be processed in parallel.
///
/// This struct holds all the data that is accessed by the systems during the
/// parallel phase of the game tick. By grouping this data together, we can

/// pass it to the parallel processing threads without needing to pass the
/// entire `Game` struct.
pub struct ParallelGameState {
    /// The game world, including the grid, biomes, and resources.
    pub map: Map,
    /// The ECS world, which manages all entities and their components.
    pub world: Arc<Mutex<World>>,
    /// The registry for all items in the simulation.
    pub item_registry: ItemRegistry,
    /// The manager for all crafting recipes.
    pub recipe_manager: Arc<RecipeManager>,
    /// The event bus for communication between systems.
    pub event_bus: Arc<Mutex<EventBus>>,
}
