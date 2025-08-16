use std::collections::HashMap;
use super::map::Map;
use super::player::Player;
use super::brain::{Brain, HighLevelState};
use super::recipes::RecipeManager;
use super::errors::SimulationError;
use super::actions::{Action, Direction, get_all_actions};
use super::item::ItemRegistry;
use super::entity::Entity;
use super::animal::Animal;
use super::dropped_item::DroppedItem;
use std::any::Any;
use std::sync::{Arc, Mutex};
use tokio::task;

use rand::Rng;

use super::config::*;


pub struct Game {
    pub map: Map,
    pub entities: Vec<Box<dyn Entity>>,
    pub item_registry: ItemRegistry,
    pub recipe_manager: Arc<RecipeManager>,
    next_instance_id: u32,
}

impl Game {
    pub fn new() -> Self {
        let map = Map::new(WIDTH, HEIGHT).expect("Failed to create map");
        let item_registry = ItemRegistry::new("items.json");
        let recipe_manager = Arc::new(RecipeManager::new("recipes.json"));

        let mut entities: Vec<Box<dyn Entity>> = Vec::new();
        let actions = get_all_actions();

        for i in 0..NUM_PLAYERS {
            entities.push(Box::new(Player::new(i as u32, 0, 0)));
        }

        entities.push(Box::new(Animal {
            id: 100,
            x: 5,
            y: 5,
            health: 50,
            species: "deer".to_string(),
        }));

        Game {
            map,
            entities,
            item_registry,
            recipe_manager,
            next_instance_id: 0,
        }
    }

    fn _is_adjacent_to(&self, px: u32, py: u32, tile_type: char) -> bool {
        for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let nx = (px as i32 + dx) as u32;
            let ny = (py as i32 + dy) as u32;
            if nx < self.map.width && ny < self.map.height {
                if self.map.grid[ny as usize][nx as usize].tile_type == tile_type {
                    return true;
                }
            }
        }
        false
    }

    fn _find_adjacent_tile(&self, px: u32, py: u32, tile_type: char) -> Option<(u32, u32)> {
        for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let nx = (px as i32 + dx) as u32;
            let ny = (py as i32 + dy) as u32;
            if nx < self.map.width && ny < self.map.height {
                if self.map.grid[ny as usize][nx as usize].tile_type == tile_type {
                    return Some((nx, ny));
                }
            }
        }
        None
    }

    fn _find_adjacent_player(&self, player_index: usize) -> Option<usize> {
        let (px, py) = self.entities[player_index].get_position();
        for i in 0..self.entities.len() {
            if i != player_index {
                let other_player = &self.entities[i];
                let (other_px, other_py) = other_player.get_position();
                if (other_px == px && (other_py == py + 1 || other_py == py.wrapping_sub(1))) ||
                   (other_py == py && (other_px == px + 1 || other_px == px.wrapping_sub(1))) {
                    if other_player.is_alive() {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    fn _find_and_set_valid_start_positions(&mut self) {
        let mut rng = rand::thread_rng();
        let mut occupied_positions = std::collections::HashSet::new();

        let plains_biome = self.map.biomes.iter().find(|b| b.name == "plains");
        let plains_tile_type = plains_biome.map_or('.', |b| b.tile_type);

        for entity in &mut self.entities {
            loop {
                let x = rng.gen_range(0..self.map.width);
                let y = rng.gen_range(0..self.map.height);
                if self.map.grid[y as usize][x as usize].tile_type == plains_tile_type && !occupied_positions.contains(&(x, y)) {
                    // This is a bit of a hack, but we need to set the position of the entity.
                    // We can't do this through the Entity trait, so we need to downcast.
                    let any_entity = entity.as_any();
                    if let Some(player) = any_entity.downcast_mut::<Player>() {
                        player.x = x;
                        player.y = y;
                    } else if let Some(animal) = any_entity.downcast_mut::<Animal>() {
                        animal.x = x;
                        animal.y = y;
                    }
                    occupied_positions.insert((x, y));
                    break;
                }
            }
        }
    }

    fn setup_new_map(&mut self) {
        self.map.generate_island_map(25.0, 5, 0.5, 2.0);
        let mut rng = rand::thread_rng();

        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let tile = &self.map.grid[y as usize][x as usize];
                for resource in &self.map.resources {
                    if resource.biomes.contains(&tile.biome) {
                        if rng.r#gen::<f64>() < resource.density {
                            self.map.add_resource(x, y, resource.tile_type);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub async fn run(&mut self) -> Result<(), SimulationError> {
        println!("--- Starting Rust Training Simulation ---");
        self.setup_new_map();
        self._find_and_set_valid_start_positions();

        println!("Initial Map:");
        self.map.display(&self.entities);

        for episode in 0..EPISODES {
            self._respawn_resources(episode);
            for entity in &mut self.entities {
                let any_entity = entity.as_any();
                if let Some(player) = any_entity.downcast_mut::<Player>() {
                    player.reset();
                }
            }
            self._find_and_set_valid_start_positions();

            for _step in 0..MAX_STEPS_PER_EPISODE {
                let mut actions = Vec::new();
                for i in 0..self.entities.len() {
                    if self.entities[i].is_alive() {
                        let action = self.entities[i].update(self)?;
                        actions.push((i, action));
                    }
                }

                for (i, action) in actions {
                    if let Some(action) = action {
                        self._perform_action(i, &action, episode);
                    }
                }
            }

            if (episode + 1) % 200 == 0 {
                println!("Episode {}/{}", episode + 1, EPISODES);
            }
        }

        println!("--- Training Finished ---");
        Ok(())
    }

    fn _respawn_resources(&mut self, current_episode: u32) {
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let tile = &mut self.map.grid[y as usize][x as usize];
                if let Some(depletion_episode) = tile.depletion_episode {
                    if current_episode >= depletion_episode + 4 {
                        // Find the original biome tile type
                        for biome in &self.map.biomes {
                            if biome.name == tile.biome {
                                tile.tile_type = biome.tile_type;
                                break;
                            }
                        }
                        tile.remaining_resources = Some(5);
                        tile.depletion_episode = None;
                    }
                }
            }
        }
    }

    fn _display_mental_map(&self, player_index: usize) {
        println!("--- Player {} Mental Map ---", player_index);
        // let brain = self.brains[player_index].lock().unwrap();
        // for y in 0..HEIGHT {
        //     for x in 0..WIDTH {
        //         match &brain.mental_map[y as usize][x as usize] {
        //             Some(memory_tile) => print!("{} ", memory_tile.tile.tile_type),
        //             None => print!("? "),
        //         }
        //     }
        //     println!();
        // }
        println!("--------------------------");
    }

    fn _handle_equip_action(&mut self, player_index: usize, item: &str) -> f64 {
        if let Some(player) = self.entities[player_index].as_any().downcast_mut::<Player>() {
            if player.get_total_quantity(item) > 0 {
                player.held_item = Some(item.to_string());
                2.0
            } else {
                -2.0
            }
        } else {
            -1.0 // Not a player
        }
    }

    fn _handle_craft_action(&mut self, player_index: usize, item: &str) -> f64 {
        let required_resources = self.recipe_manager.get_required_resources(item, 1);

        if !required_resources.is_empty() && required_resources.get(item).is_none() {
            if let Some(player) = self.entities[player_index].as_any().downcast_mut::<Player>() {
                if player.has_resources(&required_resources) {
                    if player.remove_resources(&required_resources) {
                        if item == "lock" {
                            let lock_id = self.next_instance_id;
                            self.next_instance_id += 1;
                            let key_id = lock_id;

                            if player.add_item("lock", 1, Some(lock_id), &self.item_registry) &&
                               player.add_item("key", 1, Some(key_id), &self.item_registry) {
                                50.0
                            } else {
                                -15.0
                            }
                        } else {
                            if player.add_item(item, 1, None, &self.item_registry) {
                                50.0
                            } else { -15.0 }
                        }
                    } else { -15.0 }
                } else { -10.0 }
            } else { -1.0 }
        } else { -1.0 }
    }

    fn _handle_move_action(&mut self, player_index: usize, direction: &Direction) -> f64 {
        if let Some(player) = self.entities[player_index].as_any().downcast_mut::<Player>() {
            let direction_str = match direction {
                Direction::Up => "up",
                Direction::Down => "down",
                Direction::Left => "left",
                Direction::Right => "right",
            };
            if player.move_player(direction_str, &self.map, &self.entities) {
                let (new_px, new_py) = (player.x, player.y);
                let current_tile = &self.map.grid[new_py as usize][new_px as usize];
                if current_tile.tile_type == 'M' { -2.0 }
                else if "RUIT".contains(current_tile.tile_type) { 1.0 }
                else { 0.0 }
            } else { -5.0 }
        } else {
            -1.0
        }
    }

    fn _handle_gather_action(&mut self, player_index: usize, px: u32, py: u32, episode: u32) -> f64 {
        let (tile_type, biome_name) = {
            let tile = &self.map.grid[py as usize][px as usize];
            (tile.tile_type, tile.biome.clone())
        };

        if self.map.grid[py as usize][px as usize].remaining_resources.is_none() || self.map.grid[py as usize][px as usize].remaining_resources == Some(0) {
            return -2.0; // Nothing to gather
        }

        if let Some(player) = self.entities[player_index].as_any().downcast_mut::<Player>() {
            let held_item_name = player.held_item.as_deref();
            let held_item = held_item_name.and_then(|name| self.item_registry.get_item(name));

        if let Some(item) = held_item {
            if let Some(properties) = &item.properties {
                let efficiency = properties.get("efficiency").cloned().unwrap_or(1.0) as u32;
                let required_tool = match tile_type {
                    'T' => "stone_axe",
                    'R' => "stone_pickaxe",
                    'U' => "stone_pickaxe",
                    'I' => "metal_pickaxe",
                    _ => "",
                };

                if item.name == required_tool {
                    let resource = match tile_type {
                        'T' => "wood",
                        'R' => "stone",
                        'U' => "sulfur",
                        'I' => "iron_ore",
                        _ => "",
                    };

                    if player.add_item(resource, efficiency, None, &self.item_registry) {
                        if let Some(res) = &mut self.map.grid[py as usize][px as usize].remaining_resources {
                            *res -= 1;
                            if *res == 0 {
                                // Find the original biome tile type
                                for biome in &self.map.biomes {
                                    if biome.name == biome_name {
                                        self.map.grid[py as usize][px as usize].tile_type = biome.tile_type;
                                        break;
                                    }
                                }
                                self.map.grid[py as usize][px as usize].depletion_episode = Some(episode);
                            }
                        }

                        if let Some(slot_index) = player.inventory.iter().position(|s| s.is_some() && s.as_ref().unwrap().item == item.name) {
                            if let Some(slot) = &mut player.inventory[slot_index] {
                                if let Some(durability) = &mut slot.durability {
                                    *durability -= 1.0;
                                    if *durability <= 0.0 {
                                        player.inventory[slot_index] = None;
                                    }
                                }
                            }
                        }

                        return 20.0;
                    } else { return -15.0; }
                } else { return -10.0; }
            } else { return -10.0; }
        } else { return -10.0; }
        }
        -1.0
    }

    fn _handle_place_furnace_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        if let Some(player) = self.entities[player_index].as_any().downcast_mut::<Player>() {
            if player.get_total_quantity("furnace") > 0 && self.map.grid[py as usize][px as usize].tile_type == '.' {
                // Check for other entities
                for entity in &self.entities {
                    if entity.get_id() != player.id {
                        let (ex, ey) = entity.get_position();
                        if ex == px && ey == py {
                            return -5.0; // Another entity is in the way
                        }
                    }
                }
                let mut recipe = HashMap::new(); recipe.insert("furnace".to_string(), 1);
                player.remove_resources(&recipe);
                self.map.grid[py as usize][px as usize].tile_type = 'F';
                40.0
            } else { -5.0 }
        } else { -1.0 }
    }

    fn _handle_place_door_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        if let Some(player) = self.entities[player_index].as_any().downcast_mut::<Player>() {
            if player.get_total_quantity("door") > 0 && self.map.grid[py as usize][px as usize].tile_type == 'O' {
                // Check for other entities
                for entity in &self.entities {
                    if entity.get_id() != player.id {
                        let (ex, ey) = entity.get_position();
                        if ex == px && ey == py {
                            return -5.0; // Another entity is in the way
                        }
                    }
                }
                let mut recipe = HashMap::new(); recipe.insert("door".to_string(), 1);
                player.remove_resources(&recipe);
                self.map.grid[py as usize][px as usize].tile_type = 'D';
                40.0
            } else { -5.0 }
        } else { -1.0 }
    }

    fn _handle_smelt_iron_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        let mut recipe = HashMap::new();
        recipe.insert("iron_ore".to_string(), 1);
        recipe.insert("wood".to_string(), 1);

        if self._is_adjacent_to(px, py, 'F') {
            if let Some(player) = self.entities[player_index].as_any().downcast_mut::<Player>() {
                if player.has_resources(&recipe) {
                    if player.remove_resources(&recipe) {
                        player.add_item("iron_bars", 1, None, &self.item_registry);
                        60.0
                    } else { -15.0 }
                } else { -12.0 }
            } else { -1.0 }
        } else { -12.0 }
    }

    fn _handle_build_action(&mut self, player_index: usize, structure: &str, px: u32, py: u32) -> f64 {
        if let Some(player) = self.entities[player_index].as_any().downcast_mut::<Player>() {
            if player.get_total_quantity(structure) > 0 {
                // Check for other entities
                for entity in &self.entities {
                    if entity.get_id() != player.id {
                        let (ex, ey) = entity.get_position();
                        if ex == px && ey == py {
                            return -5.0; // Another entity is in the way
                        }
                    }
                }

                let current_tile = &mut self.map.grid[py as usize][px as usize];
                let item = self.item_registry.get_item(structure);
                let health = item.and_then(|i| i.properties.as_ref()).and_then(|p| p.get("health")).cloned().unwrap_or(100.0);

                let tile_char = match structure {
                    "foundation" => 'B',
                    "wall" => '#',
                    "doorway" => 'O',
                    _ => 'X',
                };

                if tile_char != 'X' {
                    current_tile.tile_type = tile_char;
                    current_tile.health = Some(health);
                    let mut recipe = HashMap::new();
                    recipe.insert(structure.to_string(), 1);
                    player.remove_resources(&recipe);
                    30.0
                } else { -0.1 }
            } else { -5.0 }
        } else { -1.0 }
    }

    fn _handle_attach_lock_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        let has_lock = if let Some(player) = self.entities[player_index].as_any().downcast_ref::<Player>() {
            player.has_lock()
        } else {
            false
        };

        if !has_lock {
            return -10.0;
        }

        if let Some((door_x, door_y)) = self._find_adjacent_tile(px, py, 'D') {
            let door_tile = &mut self.map.grid[door_y as usize][door_x as usize];
            if door_tile.lock_id.is_none() {
                if let Some(player) = self.entities[player_index].as_any().downcast_mut::<Player>() {
                    if let Some(lock_id) = player.find_and_remove_lock() {
                        door_tile.tile_type = 'L';
                        door_tile.lock_id = Some(lock_id);
                        return 50.0;
                    }
                }
            }
        }
        -5.0
    }

    fn _handle_open_door_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        if let Some((door_x, door_y)) = self._find_adjacent_tile(px, py, 'D') {
            self.map.grid[door_y as usize][door_x as usize].tile_type = 'd';
            return 10.0;
        }

        if let Some((door_x, door_y)) = self._find_adjacent_tile(px, py, 'L') {
            let door_tile = &self.map.grid[door_y as usize][door_x as usize];
            if let Some(lock_id) = door_tile.lock_id {
                if let Some(player) = self.entities[player_index].as_any().downcast_ref::<Player>() {
                    if player.has_key(lock_id) {
                        self.map.grid[door_y as usize][door_x as usize].tile_type = 'd';
                        return 20.0;
                    } else {
                        return -15.0;
                    }
                }
            }
        }
        -5.0
    }

    fn _handle_close_door_action(&mut self, _player_index: usize, px: u32, py: u32) -> f64 {
        if let Some((door_x, door_y)) = self._find_adjacent_tile(px, py, 'd') {
            self.map.grid[door_y as usize][door_x as usize].tile_type = 'D';
            10.0
        } else {
            -5.0
        }
    }

    fn _handle_structural_collapse(&mut self, x: u32, y: u32) {
        println!("Structural collapse at ({}, {})", x, y);
        let mut to_check = vec![(x, y)];
        let mut checked = std::collections::HashSet::new();

        while let Some((cx, cy)) = to_check.pop() {
            if !checked.insert((cx, cy)) {
                continue;
            }

            for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let nx = (cx as i32 + dx) as u32;
                let ny = (cy as i32 + dy) as u32;

                if nx < self.map.width && ny < self.map.height && !checked.contains(&(nx, ny)) {
                    let is_supported = if ny > 0 {
                        let (_grid_slice_above, grid_slice_below) = self.map.grid.split_at_mut(ny as usize);
                        let below_tile = &grid_slice_below[0][nx as usize];
                        "B#".contains(below_tile.tile_type)
                    } else { false };

                    let tile = &mut self.map.grid[ny as usize][nx as usize];
                    if "B#DO".contains(tile.tile_type) && !is_supported {
                        tile.tile_type = '.';
                        tile.health = None;
                        to_check.push((nx, ny));
                    }
                }
            }
        }
    }

    fn _handle_attack_action(&mut self, player_index: usize) -> f64 {
        let attacker_id = self.entities[player_index].get_id();
        let held_item_name = if let Some(player) = self.entities[player_index].as_any().downcast_ref::<Player>() {
            player.held_item.clone()
        } else {
            None
        };

        let damage = if let Some(item_name) = &held_item_name {
            self.item_registry.get_item(item_name).and_then(|item| item.properties.as_ref()).and_then(|p| p.get("damage")).cloned().unwrap_or(1.0)
        } else {
            1.0
        };

        if let Some(other_player_index) = self._find_adjacent_player(player_index) {
            let mut new_entities = Vec::new();
            let victim = &mut self.entities[other_player_index];
            let victim_pos = victim.get_position();

            if let Some(player) = victim.as_any().downcast_mut::<Player>() {
                player.health -= damage as i32;
                if player.health <= 0 {
                    // Drop inventory
                    for slot in &player.inventory {
                        if let Some(s) = slot {
                            new_entities.push(Box::new(DroppedItem {
                                id: self.next_instance_id,
                                x: victim_pos.0,
                                y: victim_pos.1,
                                item: s.item.clone(),
                                quantity: s.quantity,
                            }) as Box<dyn Entity>);
                            self.next_instance_id += 1;
                        }
                    }
                    self.entities.extend(new_entities);
                    return 100.0;
                }
            } else if let Some(animal) = victim.as_any().downcast_mut::<Animal>() {
                animal.health -= damage as i32;
                if animal.health <= 0 {
                    // Drop meat
                    new_entities.push(Box::new(DroppedItem {
                        id: self.next_instance_id,
                        x: victim_pos.0,
                        y: victim_pos.1,
                        item: "meat".to_string(),
                        quantity: 1,
                    }) as Box<dyn Entity>);
                    self.next_instance_id += 1;
                    self.entities.extend(new_entities);
                    return 50.0;
                }
            }
            return 10.0;
        }
        -1.0
    }

    fn _handle_pickup_action(&mut self, player_index: usize) -> f64 {
        let player_pos = self.entities[player_index].get_position();
        let mut items_to_pickup = Vec::new();

        for (i, entity) in self.entities.iter().enumerate() {
            if let Some(item) = entity.as_any().downcast_ref::<DroppedItem>() {
                if item.get_position() == player_pos {
                    items_to_pickup.push(i);
                }
            }
        }

        if items_to_pickup.is_empty() {
            return -1.0;
        }

        for i in items_to_pickup.iter().rev() {
            let item_entity = self.entities.remove(*i);
            if let Some(item) = item_entity.as_any().downcast_ref::<DroppedItem>() {
                if let Some(player) = self.entities[player_index].as_any().downcast_mut::<Player>() {
                    player.add_item(&item.item, item.quantity, None, &self.item_registry);
                }
            }
        }

        20.0
    }

    pub fn _perform_action(&mut self, player_index: usize, action: &Action, episode: u32) -> f64 {
        let (px, py) = self.entities[player_index].get_position();

        match action {
            Action::Move(direction) => self._handle_move_action(player_index, direction),
            Action::Gather => self._handle_gather_action(player_index, px, py, episode),
            Action::Craft(item) => self._handle_craft_action(player_index, item),
            Action::Equip(item) => self._handle_equip_action(player_index, item),
            Action::Place(item) => {
                if item == "furnace" {
                    self._handle_place_furnace_action(player_index, px, py)
                } else if item == "door" {
                    self._handle_place_door_action(player_index, px, py)
                } else {
                    -0.1
                }
            },
            Action::Smelt => self._handle_smelt_iron_action(player_index, px, py),
            Action::Build(structure) => self._handle_build_action(player_index, structure, px, py),
            Action::Open => self._handle_open_door_action(player_index, px, py),
            Action::Close => self._handle_close_door_action(player_index, px, py),
            Action::AttachLock => self._handle_attach_lock_action(player_index, px, py),
            Action::Attack => self._handle_attack_action(player_index),
            Action::Pickup => self._handle_pickup_action(player_index),
        }
    }
}

