use crate::ecs::World;
use crate::components::{Position, Velocity, WantsToGather, Resource, WantsToCraft, WantsToBuild, WantsToAttack, Health, DroppedItem, WantsToPickup};
use crate::player::Player;
use crate::recipes::RecipeManager;
use crate::item::ItemRegistry;
use crate::map::{Map, TileState};
use crate::events::{EventBus, Event};
use crate::fov;
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

pub fn movement_system(world: &mut World, map: &mut Map) {
    let entities_with_velocity: Vec<_> = world.entities.iter().filter_map(|&entity| {
        world.get_component::<Velocity>(entity).map(|vel| (entity, *vel))
    }).collect();

    for (entity, vel) in entities_with_velocity {
        if let Some(pos) = world.get_component_mut::<Position>(entity) {
            // Remove from old position in spatial map
            map.spatial_map.entry((pos.x, pos.y)).and_modify(|v| v.retain(|&e| e != entity));

            pos.x = (pos.x as i32 + vel.dx) as u32;
            pos.y = (pos.y as i32 + vel.dy) as u32;

            // Add to new position in spatial map
            map.spatial_map.entry((pos.x, pos.y)).or_default().push(entity);
        }
    }

    // Reset velocities
    let entities_with_velocity: Vec<_> = world.entities.iter().copied().collect();
    for entity in entities_with_velocity {
        world.remove_component::<Velocity>(entity);
    }
}

pub fn gathering_system(world: &mut World, item_registry: &ItemRegistry) {
    let mut to_gather = Vec::new();
    for entity in 0..world.entities.len() {
        if let Some(wants_to_gather) = world.get_component::<WantsToGather>(entity) {
            to_gather.push((entity, wants_to_gather.target));
        }
    }

    for (gatherer, target) in to_gather {
        if let (Some(gatherer_pos), Some(target_pos)) = (
            world.get_component::<Position>(gatherer).map(|p| *p),
            world.get_component::<Position>(target).map(|p| *p),
        ) {
            let dx = (gatherer_pos.x as i32 - target_pos.x as i32).abs();
            let dy = (gatherer_pos.y as i32 - target_pos.y as i32).abs();

            if dx <= 1 && dy <= 1 {
                let resource_name = if let Some(resource) = world.get_component_mut::<Resource>(target) {
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
                    if let Some(player) = world.get_component_mut::<Player>(gatherer) {
                        player.add_item(&name, 1, None, item_registry);
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

pub fn crafting_system(world: &mut World, recipe_manager: &RecipeManager, item_registry: &ItemRegistry) {
    let mut to_craft = Vec::new();
    for entity in 0..world.entities.len() {
        if let Some(wants_to_craft) = world.get_component::<WantsToCraft>(entity) {
            to_craft.push((entity, wants_to_craft.clone()));
        }
    }

    for (crafter, wants_to_craft) in to_craft {
        let required_resources = recipe_manager.get_required_resources(&wants_to_craft.item_name, 1);
        if let Some(player) = world.get_component_mut::<Player>(crafter) {
            if player.has_resources(&required_resources) {
                if player.remove_resources(&required_resources) {
                    player.add_item(&wants_to_craft.item_name, 1, None, item_registry);
                }
            }
        }
    }

    // Reset wants to craft
    for entity in 0..world.entities.len() {
        world.remove_component::<WantsToCraft>(entity);
    }
}

pub fn building_system(world: &mut World, map: &mut Map) {
    let mut to_build = Vec::new();
    for entity in 0..world.entities.len() {
        if let Some(wants_to_build) = world.get_component::<WantsToBuild>(entity) {
            to_build.push((entity, wants_to_build.clone()));
        }
    }

    for (builder, wants_to_build) in to_build {
        if let Some(builder_pos) = world.get_component::<Position>(builder).map(|p| *p) {
            let tile = &mut map.grid[builder_pos.y as usize][builder_pos.x as usize];

            if tile.tile_type == '.' {
                if let Some(player) = world.get_component_mut::<Player>(builder) {
                    if player.get_total_quantity(&wants_to_build.structure_name) > 0 {
                        let mut recipe = std::collections::HashMap::new();
                        recipe.insert(wants_to_build.structure_name.clone(), 1);
                        player.remove_resources(&recipe);

                        tile.tile_type = match wants_to_build.structure_name.as_str() {
                            "foundation" => 'B',
                            "wall" => '#',
                            "doorway" => 'O',
                            _ => 'X',
                        };
                    }
                }
            }
        }
    }

    // Reset wants to build
    for entity in 0..world.entities.len() {
        world.remove_component::<WantsToBuild>(entity);
    }
}

pub fn combat_system(world: &mut World, event_bus: &Arc<Mutex<EventBus>>) {
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
            event_bus.lock().expect("Failed to lock event bus").publish(Event::EntityDied(target));
        }
    }

    // Reset wants to attack
    for entity in 0..world.entities.len() {
        world.remove_component::<WantsToAttack>(entity);
    }
}

pub fn pickup_system(world: &mut World, item_registry: &ItemRegistry, map: &mut Map) {
    let mut to_pickup = Vec::new();
    for entity in world.entities.clone() {
        if world.get_component::<WantsToPickup>(entity).is_some() {
            to_pickup.push(entity);
        }
    }

    for picker_upper in to_pickup {
        if let Some(picker_upper_pos) = world.get_component::<Position>(picker_upper).map(|p| *p) {
            let mut items_to_remove = Vec::new();
            let mut items_to_add = Vec::new();

            if let Some(entities_on_tile) = map.spatial_map.get(&(picker_upper_pos.x, picker_upper_pos.y)) {
                for &entity in entities_on_tile {
                    if let Some(item) = world.get_component::<DroppedItem>(entity) {
                        items_to_add.push((picker_upper, item.clone()));
                        items_to_remove.push(entity);
                    }
                }
            }


            for (picker_upper, item) in items_to_add {
                if let Some(player) = world.get_component_mut::<Player>(picker_upper) {
                    player.add_item(&item.item_name, item.quantity, None, item_registry);
                }
            }

            for entity in items_to_remove.iter() {
                if let Some(pos) = world.get_component::<Position>(*entity) {
                    map.spatial_map.entry((pos.x, pos.y)).and_modify(|v| v.retain(|&e| e != *entity));
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

pub fn death_system(world: &mut World, event_bus: &Arc<Mutex<EventBus>>, map: &mut Map) {
    let events = event_bus.lock().expect("Failed to lock event bus").take_events();
    for event in events {
        match event {
            Event::EntityDied(entity) => {
                if let Some(pos) = world.get_component::<Position>(entity) {
                    // Remove from spatial map
                    map.spatial_map.entry((pos.x, pos.y)).and_modify(|v| v.retain(|&e| e != entity));

                    // Turn the dead entity into a dropped item (meat)
                    world.add_component(entity, DroppedItem {
                        item_name: "meat".to_string(),
                        quantity: 1,
                    });
                }
                world.remove_entity(entity);
            }
        }
    }
}
