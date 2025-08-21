use crate::ecs::World;
use crate::errors::SimulationError;
use crate::events::EventBus;
use crate::item::ItemRegistry;
use crate::map::Map;
use crate::recipes::RecipeManager;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub mod brain_event_handler;
pub mod building;
pub mod combat;
pub mod crafting;
pub mod death;
pub mod gathering;
pub mod movement;
pub mod pickup;
pub mod storage;
pub mod visibility;

/// Represents a shared resource that a system can access.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Resource {
    Map,
    World,
    ItemRegistry,
    RecipeManager,
    EventBus,
}

/// A collection of resources that are passed to systems when they are run.
/// This is constructed by the scheduler, which provides the necessary components
/// from the main `ParallelGameState`.
pub struct SystemResources<'a> {
    pub map: &'a mut Map,
    pub item_registry: &'a ItemRegistry,
    pub recipe_manager: &'a Arc<RecipeManager>,
    pub event_bus: &'a Arc<Mutex<EventBus>>,
    pub is_day: bool,
}

use crate::systems::brain_event_handler::BrainEventHandlerSystem;
use crate::systems::building::BuildingSystem;
use crate::systems::combat::CombatSystem;
use crate::systems::crafting::CraftingSystem;
use crate::systems::death::DeathSystem;
use crate::systems::gathering::GatheringSystem;
use crate::systems::movement::MovementSystem;
use crate::systems::pickup::PickupSystem;
use crate::systems::storage::StorageSystem;
use crate::systems::visibility::VisibilitySystem;

/// The core trait for any system in the simulation.
///
/// Each system must declare the resources it reads from and writes to. This
/// information is used by the scheduler to determine which systems can be run
/// in parallel without causing data races or other concurrency issues.
pub trait System: Send + Sync {
    /// Returns the name of the system for debugging and logging.
    fn name(&self) -> &'static str;

    /// Declares the set of resources the system will read from.
    fn read_resources(&self) -> HashSet<Resource> {
        HashSet::new()
    }

    /// Declares the set of resources the system will write to.
    fn write_resources(&self) -> HashSet<Resource> {
        HashSet::new()
    }

    /// Executes the system's logic.
    ///
    /// The scheduler provides mutable access to the `World` (ECS) and a
    /// `SystemResources` struct containing other shared game state.
    fn run(&self, world: &mut World, resources: &mut SystemResources) -> Result<(), SimulationError>;
}

use crate::world::ParallelGameState;

pub struct Scheduler {
    systems: Vec<Box<dyn System>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            systems: vec![
                Box::new(VisibilitySystem),
                Box::new(MovementSystem),
                Box::new(GatheringSystem),
                Box::new(CraftingSystem),
                Box::new(BuildingSystem),
                Box::new(StorageSystem),
                Box::new(CombatSystem),
                Box::new(PickupSystem),
                Box::new(DeathSystem),
                Box::new(BrainEventHandlerSystem),
            ],
        }
    }

    pub fn run_parallel(
        &self,
        state: &mut ParallelGameState,
        is_day: bool,
    ) -> Result<(), SimulationError> {
        // For now, run systems sequentially to ensure correctness before parallelizing.
        let mut world = state
            .world
            .lock()
            .map_err(|e| SimulationError::MutexLockError(e.to_string()))?;

        let mut resources = SystemResources {
            map: &mut state.map,
            item_registry: &state.item_registry,
            recipe_manager: &state.recipe_manager,
            event_bus: &state.event_bus,
            is_day,
        };

        for system in &self.systems {
            system.run(&mut world, &mut resources)?;
        }

        Ok(())
    }
}
