use crate::ecs::World;
use crate::components::{Position, Velocity, WantsToGather, Resource, WantsToCraft, WantsToBuild};
use crate::player::Player;
use crate::recipes::RecipeManager;
use crate::item::ItemRegistry;
use crate::map::Map;

pub fn movement_system(world: &mut World) {
    for entity in 0..world.entities.len() {
        let (dx, dy) = if let Some(vel) = world.get_component::<Velocity>(entity) {
            (vel.dx, vel.dy)
        } else {
            (0, 0)
        };

        if dx != 0 || dy != 0 {
            if let Some(pos) = world.get_component_mut::<Position>(entity) {
                pos.x = (pos.x as i32 + dx) as u32;
                pos.y = (pos.y as i32 + dy) as u32;
            }
        }
    }

    // Reset velocities
    for entity in 0..world.entities.len() {
        world.remove_component::<Velocity>(entity);
    }
}

pub fn gathering_system(world: &mut World) {
    let mut to_gather = Vec::new();
    for entity in 0..world.entities.len() {
        if let Some(wants_to_gather) = world.get_component::<WantsToGather>(entity) {
            to_gather.push((entity, wants_to_gather.target));
        }
    }

    for (gatherer, target) in to_gather {
        let gatherer_pos = world.get_component::<Position>(gatherer).unwrap();
        let target_pos = world.get_component::<Position>(target).unwrap();

        let dx = (gatherer_pos.x as i32 - target_pos.x as i32).abs();
        let dy = (gatherer_pos.y as i32 - target_pos.y as i32).abs();

        if dx <= 1 && dy <= 1 {
            let resource_type = if let Some(resource) = world.get_component_mut::<Resource>(target) {
                if resource.quantity > 0 {
                    resource.quantity -= 1;
                    Some(resource.resource_type)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(resource_type) = resource_type {
                if let Some(player) = world.get_component_mut::<Player>(gatherer) {
                    // This is a placeholder for adding the item to the inventory
                    println!("Player {} gathered 1 of {}", player.id, resource_type);
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
        let builder_pos = world.get_component::<Position>(builder).unwrap();
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

    // Reset wants to build
    for entity in 0..world.entities.len() {
        world.remove_component::<WantsToBuild>(entity);
    }
}
