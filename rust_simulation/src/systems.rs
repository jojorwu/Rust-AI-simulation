use crate::components::{
    Chest, DroppedItem, Health, Inventory, Position, Resource, Velocity, WantsToAttack,
    WantsToBuild, WantsToCraft, WantsToGather, WantsToPickup, WantsToStoreItem,
};
use crate::ecs::World;
use crate::events::{Event, EventBus};
use crate::fov;
use crate::item::ItemRegistry;
use crate::map::{Map, TileState};
use crate::player::Player;
use crate::recipes::RecipeManager;
use std::sync::{Arc, Mutex};

pub fn visibility_system(world: &mut World, map: &Map, is_day: bool) {
    for entity in 0..world.entities.len() {
        // We need both a position and a player component.
        // We can't get mutable access to player and then immutable access to position in the same loop iteration easily.
        // So we get the position first.
        let player_pos = match world.get_component::<Position>(entity) {
            Some(pos) => *pos,
            None => continue,
        };

        if let Some(player) = world.get_component_mut::<Player>(entity) {
            // Step 1: Set all currently visible tiles to explored.
            for y in 0..player.mental_map.height {
                for x in 0..player.mental_map.width {
                    if player.mental_map.grid[y as usize][x as usize] == TileState::Visible {
                        player.mental_map.grid[y as usize][x as usize] = TileState::Explored;
                    }
                }
            }

            // Step 2: Calculate the new field of view.
            let fov_radius = if is_day { 8 } else { 4 };
            let visible_tiles = fov::field_of_view(&player_pos, fov_radius, map);

            // Step 3: Mark all tiles in the FOV as visible.
            for pos in visible_tiles.iter() {
                player.mental_map.grid[pos.y as usize][pos.x as usize] = TileState::Visible;
            }
        }
    }
}

pub fn storage_system(world: &mut World) {
    let mut to_store = Vec::new();
    for entity in 0..world.entities.len() {
        if let Some(wants_to_store) = world.get_component::<WantsToStoreItem>(entity) {
            to_store.push((entity, wants_to_store.clone()));
        }
    }

    let mut successful_transfers = Vec::new();

    // Step 1: Check for validity and remove from storer
    for (storer, wants_to_store) in &to_store {
        if let Some(storer_inventory) = world.get_component_mut::<Inventory>(*storer) {
            if storer_inventory.remove_item(&wants_to_store.item_name, wants_to_store.quantity) {
                // If removal was successful, queue the item for addition to the chest
                successful_transfers.push(wants_to_store.clone());
            }
        }
    }

    // Step 2: Add to chest
    for transfer in successful_transfers {
        if let Some(chest_component) = world.get_component_mut::<Chest>(transfer.target_chest) {
            chest_component
                .inventory
                .add_item(&transfer.item_name, transfer.quantity);
        }
    }

    // Reset wants to store
    for (storer, _) in to_store {
        world.remove_component::<WantsToStoreItem>(storer);
    }
}

pub fn movement_system(world: &mut World, map: &mut Map) {
    let entities_with_velocity: Vec<_> = world
        .entities
        .iter()
        .filter_map(|&entity| {
            world
                .get_component::<Velocity>(entity)
                .map(|vel| (entity, *vel))
        })
        .collect();

    for (entity, vel) in entities_with_velocity {
        if let Some(pos) = world.get_component_mut::<Position>(entity) {
            // Remove from old position in spatial map
            map.spatial_map
                .entry((pos.x, pos.y))
                .and_modify(|v| v.retain(|&e| e != entity));

            pos.x = (pos.x as i32 + vel.dx) as u32;
            pos.y = (pos.y as i32 + vel.dy) as u32;

            // Add to new position in spatial map
            map.spatial_map
                .entry((pos.x, pos.y))
                .or_default()
                .push(entity);
        }
    }

    // Reset velocities
    let entities_with_velocity: Vec<_> = world.entities.to_vec();
    for entity in entities_with_velocity {
        world.remove_component::<Velocity>(entity);
    }
}

pub fn gathering_system(world: &mut World, _item_registry: &ItemRegistry) {
    let mut to_gather = Vec::new();
    for entity in 0..world.entities.len() {
        if let Some(wants_to_gather) = world.get_component::<WantsToGather>(entity) {
            to_gather.push((entity, wants_to_gather.target));
        }
    }

    for (gatherer, target) in to_gather {
        if let (Some(gatherer_pos), Some(target_pos)) = (
            world.get_component::<Position>(gatherer).copied(),
            world.get_component::<Position>(target).copied(),
        ) {
            let dx = (gatherer_pos.x as i32 - target_pos.x as i32).abs();
            let dy = (gatherer_pos.y as i32 - target_pos.y as i32).abs();

            if dx <= 1 && dy <= 1 {
                let resource_name =
                    if let Some(resource) = world.get_component_mut::<Resource>(target) {
                        if resource.quantity > 0 {
                            resource.quantity -= 1;
                            Some(resource.name.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                if let Some(name) = resource_name {
                    if let Some(inventory) = world.get_component_mut::<Inventory>(gatherer) {
                        inventory.add_item(&name, 1);
                    }
                }
            }
        }
    }

    // Reset wants to gather
    for entity in 0..world.entities.len() {
        world.remove_component::<WantsToGather>(entity);
    }
}

pub fn crafting_system(
    world: &mut World,
    recipe_manager: &RecipeManager,
    _item_registry: &ItemRegistry,
) {
    let mut to_craft = Vec::new();
    for entity in 0..world.entities.len() {
        if let Some(wants_to_craft) = world.get_component::<WantsToCraft>(entity) {
            to_craft.push((entity, wants_to_craft.clone()));
        }
    }

    for (crafter, wants_to_craft) in to_craft {
        let required_resources =
            recipe_manager.get_required_resources(&wants_to_craft.item_name, 1);
        if let Some(inventory) = world.get_component_mut::<Inventory>(crafter) {
            if inventory.has_resources(&required_resources)
                && inventory.remove_resources(&required_resources) {
                    inventory.add_item(&wants_to_craft.item_name, 1);
                }
        }
    }

    // Reset wants to craft
    for entity in 0..world.entities.len() {
        world.remove_component::<WantsToCraft>(entity);
    }
}

use crate::components::BrainComponent;

pub fn building_system(
    world: &mut World,
    map: &mut Map,
    event_bus: &Arc<Mutex<EventBus>>,
    recipe_manager: &Arc<RecipeManager>,
) -> Result<(), crate::errors::SimulationError> {
    let mut to_build = Vec::new();
    for entity in 0..world.entities.len() {
        if let Some(wants_to_build) = world.get_component::<WantsToBuild>(entity) {
            to_build.push((entity, wants_to_build.clone()));
        }
    }

    for (builder, wants_to_build) in &to_build {
        if let Some(builder_pos) = world.get_component::<Position>(*builder).copied() {
            let tile = &mut map.grid[builder_pos.y as usize][builder_pos.x as usize];

            if tile.tile_type == '.' {
                if let Some(inventory) = world.get_component_mut::<Inventory>(*builder) {
                    let required =
                        recipe_manager.get_required_resources(&wants_to_build.structure_name, 1);
                    if inventory.has_resources(&required)
                        && inventory.remove_resources(&required) {
                            let built_structure = wants_to_build.structure_name.clone();

                            if built_structure == "chest" {
                                let chest_entity = world.create_entity();
                                world.add_component(chest_entity, builder_pos)?;
                                world.add_component(
                                    chest_entity,
                                    Chest {
                                        inventory: Inventory::new(),
                                    },
                                )?;
                                tile.tile_type = 'C';
                            } else {
                                tile.tile_type = match built_structure.as_str() {
                                    "foundation" => 'B',
                                    "wall" => '#',
                                    "doorway" => 'O',
                                    _ => 'X',
                                };

                                if built_structure == "foundation" {
                                    event_bus
                                        .lock()
                                        .map_err(|e| {
                                            crate::errors::SimulationError::MutexLockError(
                                                e.to_string(),
                                            )
                                        })?
                                        .publish(Event::FoundationBuilt {
                                            builder: *builder,
                                            position: builder_pos,
                                        });
                                }
                            }
                        }
                }
            }
        }
    }

    // Reset wants to build
    let builders: Vec<_> = to_build.iter().map(|(e, _)| *e).collect();
    for builder in builders {
        world.remove_component::<WantsToBuild>(builder);
    }
    Ok(())
}

pub fn brain_event_handler_system(
    world: &mut World,
    event_bus: &Arc<Mutex<EventBus>>,
) -> Result<(), crate::errors::SimulationError> {
    let events = event_bus
        .lock()
        .map_err(|e| crate::errors::SimulationError::MutexLockError(e.to_string()))?
        .take_events();
    for event in events {
        if let Event::FoundationBuilt { builder, position } = event {
            if let Some(brain_component) = world.get_component_mut::<BrainComponent>(builder) {
                if brain_component.home_base.is_none() {
                    brain_component.home_base = Some(position);
                }
            }
        }
    }
    Ok(())
}

pub fn combat_system(
    world: &mut World,
    event_bus: &Arc<Mutex<EventBus>>,
) -> Result<(), crate::errors::SimulationError> {
    let mut to_attack = Vec::new();
    for entity in 0..world.entities.len() {
        if let Some(wants_to_attack) = world.get_component::<WantsToAttack>(entity) {
            to_attack.push((entity, wants_to_attack.target));
        }
    }

    for (_attacker, target) in to_attack {
        let damage = 10; // Placeholder
        let mut target_dead = false;
        if let Some(health) = world.get_component_mut::<Health>(target) {
            health.current -= damage;
            if health.current <= 0 {
                target_dead = true;
            }
        }

        if target_dead {
            event_bus
                .lock()
                .map_err(|e| crate::errors::SimulationError::MutexLockError(e.to_string()))?
                .publish(Event::EntityDied(target));
        }
    }

    // Reset wants to attack
    for entity in 0..world.entities.len() {
        world.remove_component::<WantsToAttack>(entity);
    }
    Ok(())
}

pub fn pickup_system(world: &mut World, _item_registry: &ItemRegistry, map: &mut Map) {
    let mut to_pickup = Vec::new();
    for entity in world.entities.clone() {
        if world.get_component::<WantsToPickup>(entity).is_some() {
            to_pickup.push(entity);
        }
    }

    for picker_upper in to_pickup {
        if let Some(picker_upper_pos) = world.get_component::<Position>(picker_upper).copied() {
            let mut items_to_remove = Vec::new();
            let mut items_to_add = Vec::new();

            if let Some(entities_on_tile) = map
                .spatial_map
                .get(&(picker_upper_pos.x, picker_upper_pos.y))
            {
                for &entity in entities_on_tile {
                    if let Some(item) = world.get_component::<DroppedItem>(entity) {
                        items_to_add.push((picker_upper, item.clone()));
                        items_to_remove.push(entity);
                    }
                }
            }

            for (picker_upper, item) in items_to_add {
                if let Some(inventory) = world.get_component_mut::<Inventory>(picker_upper) {
                    inventory.add_item(&item.item_name, item.quantity);
                }
            }

            for entity in items_to_remove.iter() {
                if let Some(pos) = world.get_component::<Position>(*entity) {
                    map.spatial_map
                        .entry((pos.x, pos.y))
                        .and_modify(|v| v.retain(|&e| e != *entity));
                }
                world.remove_entity(*entity);
            }
        }
    }

    // Reset wants to pickup
    for entity in world.entities.clone() {
        world.remove_component::<WantsToPickup>(entity);
    }
}

pub fn death_system(
    world: &mut World,
    event_bus: &Arc<Mutex<EventBus>>,
    map: &mut Map,
) -> Result<(), crate::errors::SimulationError> {
    let events = event_bus
        .lock()
        .map_err(|e| crate::errors::SimulationError::MutexLockError(e.to_string()))?
        .take_events();
    for event in events {
        if let Event::EntityDied(entity) = event {
            if let Some(pos) = world.get_component::<Position>(entity).copied() {
                // Remove the dead entity from the spatial map
                map.spatial_map
                    .entry((pos.x, pos.y))
                    .and_modify(|v| v.retain(|&e| e != entity));

                // Create a new entity for the dropped item
                let dropped_item_entity = world.create_entity();
                world.add_component(
                    dropped_item_entity,
                    DroppedItem {
                        item_name: "meat".to_string(),
                        quantity: 1,
                    },
                )?;
                world.add_component(dropped_item_entity, pos)?;

                // Add the new dropped item to the spatial map
                map.spatial_map
                    .entry((pos.x, pos.y))
                    .or_default()
                    .push(dropped_item_entity);
            }
            // Remove the dead entity from the world
            world.remove_entity(entity);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{
        BrainComponent, Inventory, Position, Resource, Velocity, WantsToBuild, WantsToGather,
    };
    use crate::ecs::World;
    use crate::item::ItemRegistry;
    use crate::map::Map;
    use std::sync::{Arc, Mutex};

    use crate::map::Tile;

    #[test]
    fn test_building_system_publishes_event() -> Result<(), crate::errors::SimulationError> {
        let mut world = World::new();
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| crate::errors::SimulationError::EnvVarError(e.to_string()))?;
        let mut map = Map::new(
            10,
            10,
            &format!("{manifest_dir}/data/biomes.json"),
            &format!("{manifest_dir}/data/resources.json"),
        )?;
        let event_bus = Arc::new(Mutex::new(EventBus::new()));
        let recipe_manager = Arc::new(RecipeManager::new(&format!(
            "{manifest_dir}/data/recipes.json"
        ))?);

        let builder_entity = world.create_entity();
        let build_pos = Position { x: 5, y: 5 };
        world.add_component(builder_entity, build_pos)?;
        let mut inventory = Inventory::new();
        inventory.add_item("stone", 20);
        world.add_component(builder_entity, inventory)?;
        world.add_component(
            builder_entity,
            WantsToBuild {
                structure_name: "foundation".to_string(),
            },
        )?;

        // Set the tile to be buildable
        map.grid[build_pos.y as usize][build_pos.x as usize] = Tile::new('.', "plains".to_string());

        building_system(&mut world, &mut map, &event_bus, &recipe_manager)?;

        let events = event_bus
            .lock()
            .map_err(|e| crate::errors::SimulationError::MutexLockError(e.to_string()))?
            .take_events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            Event::FoundationBuilt {
                builder: builder_entity,
                position: build_pos
            }
        );
        Ok(())
    }

    #[test]
    fn test_brain_event_handler_system_sets_home_base() -> Result<(), crate::errors::SimulationError>
    {
        let mut world = World::new();
        let event_bus = Arc::new(Mutex::new(EventBus::new()));

        let player_entity = world.create_entity();
        world.add_component(player_entity, BrainComponent::new())?;

        let build_pos = Position { x: 7, y: 8 };
        event_bus
            .lock()
            .map_err(|e| crate::errors::SimulationError::MutexLockError(e.to_string()))?
            .publish(Event::FoundationBuilt {
                builder: player_entity,
                position: build_pos,
            });

        brain_event_handler_system(&mut world, &event_bus)?;

        let brain_component = world
            .get_component::<BrainComponent>(player_entity)
            .ok_or_else(|| {
                crate::errors::SimulationError::ComponentNotFound("BrainComponent".to_string())
            })?;
        assert_eq!(brain_component.home_base, Some(build_pos));
        Ok(())
    }

    #[test]
    fn test_movement_system() -> Result<(), crate::errors::SimulationError> {
        let mut world = World::new();
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| crate::errors::SimulationError::EnvVarError(e.to_string()))?;
        let mut map = Map::new(
            10,
            10,
            &format!("{manifest_dir}/data/biomes.json"),
            &format!("{manifest_dir}/data/resources.json"),
        )?;

        let entity = world.create_entity();
        world.add_component(entity, Position { x: 5, y: 5 })?;
        world.add_component(entity, Velocity { dx: 1, dy: -1 })?;

        movement_system(&mut world, &mut map);

        let position = world.get_component::<Position>(entity).ok_or_else(|| {
            crate::errors::SimulationError::ComponentNotFound("Position".to_string())
        })?;
        assert_eq!(position.x, 6);
        assert_eq!(position.y, 4);

        assert!(world.get_component::<Velocity>(entity).is_none());
        Ok(())
    }

    #[test]
    fn test_gathering_system() -> Result<(), crate::errors::SimulationError> {
        let mut world = World::new();
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| crate::errors::SimulationError::EnvVarError(e.to_string()))?;
        let item_registry = ItemRegistry::new(&format!("{manifest_dir}/data/items.json"))?;

        // Create gatherer
        let gatherer = world.create_entity();
        world.add_component(gatherer, Position { x: 5, y: 5 })?;
        world.add_component(gatherer, Inventory::new())?;

        // Create resource
        let resource_entity = world.create_entity();
        world.add_component(resource_entity, Position { x: 5, y: 6 })?;
        world.add_component(
            resource_entity,
            Resource {
                name: "wood".to_string(),
                quantity: 5,
            },
        )?;

        // Set intention to gather
        world.add_component(
            gatherer,
            WantsToGather {
                target: resource_entity,
            },
        )?;

        gathering_system(&mut world, &item_registry);

        let inventory = world.get_component::<Inventory>(gatherer).ok_or_else(|| {
            crate::errors::SimulationError::ComponentNotFound("Inventory".to_string())
        })?;
        assert_eq!(inventory.get_quantity("wood"), 1);

        let resource = world
            .get_component::<Resource>(resource_entity)
            .ok_or_else(|| {
                crate::errors::SimulationError::ComponentNotFound("Resource".to_string())
            })?;
        assert_eq!(resource.quantity, 4);

        assert!(world.get_component::<WantsToGather>(gatherer).is_none());
        Ok(())
    }
}
