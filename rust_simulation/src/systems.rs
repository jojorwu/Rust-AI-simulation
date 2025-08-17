use crate::ecs::World;
use crate::components::{Position, Velocity, WantsToGather, Resource, WantsToCraft, WantsToBuild, WantsToAttack, Health, DroppedItem, WantsToPickup};
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

pub fn combat_system(world: &mut World) {
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
            let _target_pos = *world.get_component::<Position>(target).unwrap();
            world.add_component(target, DroppedItem {
                item_name: "meat".to_string(),
                quantity: 1,
            });
            // This is a placeholder for removing the entity
            println!("Entity {} died", target);
        }
    }

    // Reset wants to attack
    for entity in 0..world.entities.len() {
        world.remove_component::<WantsToAttack>(entity);
    }
}

pub fn pickup_system(world: &mut World, item_registry: &ItemRegistry) {
    let mut to_pickup = Vec::new();
    for entity in 0..world.entities.len() {
        if world.get_component::<WantsToPickup>(entity).is_some() {
            to_pickup.push(entity);
        }
    }

    for picker_upper in to_pickup {
        if let Some(picker_upper_pos) = world.get_component::<Position>(picker_upper).map(|p| *p) {
            let mut items_to_remove = Vec::new();
            let mut items_to_add = Vec::new();

            for (i, entity) in (0..world.entities.len()).zip(world.entities.iter()) {
                if let Some(item) = world.get_component::<DroppedItem>(*entity) {
                    if let Some(item_pos) = world.get_component::<Position>(*entity) {
                        if item_pos.x == picker_upper_pos.x && item_pos.y == picker_upper_pos.y {
                            items_to_add.push((picker_upper, item.clone()));
                            items_to_remove.push(i);
                        }
                    }
                }
            }

            for (picker_upper, item) in items_to_add {
                if let Some(player) = world.get_component_mut::<Player>(picker_upper) {
                    player.add_item(&item.item_name, item.quantity, None, item_registry);
                }
            }

            for i in items_to_remove.iter().rev() {
                world.remove_entity(*i);
            }
        }
    }

    // Reset wants to pickup
    for entity in 0..world.entities.len() {
        world.remove_component::<WantsToPickup>(entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;
    use crate::components::{Position, Velocity, WantsToGather, Resource};
    use crate::player::Player;
    use crate::item::ItemRegistry;

    #[test]
    fn test_movement_system() {
        // Test basic movement
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, Position { x: 5, y: 5 });
        world.add_component(entity, Velocity { dx: -1, dy: 1 });

        movement_system(&mut world);

        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 4);
        assert_eq!(pos.y, 6);

        // Test zero velocity
        world.add_component(entity, Velocity { dx: 0, dy: 0 });
        movement_system(&mut world);
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 4);
        assert_eq!(pos.y, 6);

        // Test movement in other directions
        world.add_component(entity, Velocity { dx: 2, dy: -3 });
        movement_system(&mut world);
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 6);
        assert_eq!(pos.y, 3);
    }

    #[test]
    fn test_gathering_system() {
        let mut world = World::new();
        let item_registry = ItemRegistry::new("items.json");

        let player_entity = world.create_entity();
        world.add_component(player_entity, Player::new(0));
        world.add_component(player_entity, Position { x: 1, y: 1 });

        let resource_entity = world.create_entity();
        world.add_component(resource_entity, Resource { name: "wood".to_string(), quantity: 5 });
        world.add_component(resource_entity, Position { x: 1, y: 2 });

        world.add_component(player_entity, WantsToGather { target: resource_entity });

        gathering_system(&mut world, &item_registry);

        let player = world.get_component::<Player>(player_entity).unwrap();
        assert_eq!(player.get_total_quantity("wood"), 1);

        let resource = world.get_component::<Resource>(resource_entity).unwrap();
        assert_eq!(resource.quantity, 4);
    }
}
