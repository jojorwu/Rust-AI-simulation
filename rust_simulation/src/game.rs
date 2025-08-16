use std::collections::HashMap;
use super::map::{Map, Tile};
use super::player::Player;
use super::brain::{Brain, HighLevelState};
use super::recipes::RecipeManager;
use super::errors::SimulationError;
use super::actions::{Action, Direction, get_all_actions};
use super::item::ItemRegistry;
use std::sync::{Arc, Mutex};
use tokio::task;

use rand::Rng;

use super::config::*;


pub struct Game {
    pub map: Map,
    pub players: Vec<Player>,
    pub brains: Vec<Arc<Mutex<Brain>>>,
    pub item_registry: ItemRegistry,
    pub recipe_manager: Arc<RecipeManager>,
    next_instance_id: u32,
}

impl Game {
    pub fn new() -> Self {
        let map = Map::new(WIDTH, HEIGHT);
        let item_registry = ItemRegistry::new("items.json");
        let recipe_manager = Arc::new(RecipeManager::new("recipes.json"));

        let mut players = Vec::new();
        let mut brains = Vec::new();
        let actions = get_all_actions();

        for i in 0..NUM_PLAYERS {
            players.push(Player::new(i as u32, 0, 0));
            brains.push(Arc::new(Mutex::new(Brain::new(actions.clone(), Arc::clone(&recipe_manager), LEARNING_RATE, DISCOUNT_FACTOR, INITIAL_EPSILON))));
        }

        Game {
            map,
            players,
            brains,
            item_registry,
            recipe_manager,
            next_instance_id: 0,
        }
    }

    fn get_high_level_state(&self, player_index: usize) -> HighLevelState {
        let player = &self.players[player_index];
        let brain_lock = self.brains[player_index].lock().unwrap();

        let num_hostile_players = brain_lock.player_memories.values().filter(|m| m.relationship == super::brain::RelationshipStatus::Hostile).count() as u32;

        HighLevelState {
            has_wood: player.get_total_quantity("wood") > 0,
            has_stone: player.get_total_quantity("stone") > 0,
            has_iron_ore: player.get_total_quantity("iron_ore") > 0,
            has_stone_axe: player.get_total_quantity("stone_axe") > 0,
            num_hostile_players,
            health_level: player.health as u32,
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
        let (px, py) = (self.players[player_index].x, self.players[player_index].y);
        for i in 0..self.players.len() {
            if i != player_index {
                let other_player = &self.players[i];
                if (other_player.x == px && (other_player.y == py + 1 || other_player.y == py.wrapping_sub(1))) ||
                   (other_player.y == py && (other_player.x == px + 1 || other_player.x == px.wrapping_sub(1))) {
                    if other_player.health > 0 {
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

        for player in &mut self.players {
            loop {
                let x = rng.gen_range(0..self.map.width);
                let y = rng.gen_range(0..self.map.height);
                if self.map.grid[y as usize][x as usize].tile_type == '.' && !occupied_positions.contains(&(x, y)) {
                    player.x = x;
                    player.y = y;
                    occupied_positions.insert((x, y));
                    break;
                }
            }
        }
    }

    fn setup_new_map(&mut self) {
        self.map.generate_island_map(25.0, 5, 0.5, 2.0);

        let mut plains_tiles = Vec::new();
        let mut mountain_tiles = Vec::new();
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                match self.map.grid[y as usize][x as usize].tile_type {
                    '.' => plains_tiles.push((x, y)),
                    'M' => mountain_tiles.push((x, y)),
                    _ => (),
                }
            }
        }

        let mut rng = rand::thread_rng();
        let tree_locations = get_random_samples(&plains_tiles, NUM_TREES as usize, &mut rng);
        for (x, y) in tree_locations { self.map.add_tree(x, y); }

        let rock_candidates = [&plains_tiles[..], &mountain_tiles[..]].concat();
        let rock_locations = get_random_samples(&rock_candidates, NUM_STONE as usize, &mut rng);
        for (x, y) in rock_locations { self.map.add_rock(x, y); }

        let sulfur_locations = get_random_samples(&rock_candidates, NUM_SULFUR as usize, &mut rng);
        for (x, y) in sulfur_locations { self.map.add_sulfur(x, y); }

        let iron_locations = get_random_samples(&mountain_tiles, NUM_IRON_ORE as usize, &mut rng);
        for (x, y) in iron_locations { self.map.add_iron_ore_node(x, y); }
    }

    pub async fn run(&mut self) -> Result<(), SimulationError> {
        println!("--- Starting Rust Training Simulation ---");
        self.setup_new_map();
        self._find_and_set_valid_start_positions();

        println!("Initial Map:");
        self.map.display(&self.players);

        for episode in 0..EPISODES {
            self._respawn_resources(episode);
            for player in &mut self.players {
                player.reset();
            }
            self._find_and_set_valid_start_positions();

            for _step in 0..MAX_STEPS_PER_EPISODE {
                let mut action_handles = Vec::new();

                for i in 0..self.players.len() {
                    if self.players[i].health > 0 {
                        let high_level_state = self.get_high_level_state(i);
                        let brain = Arc::clone(&self.brains[i]);
                        let player = self.players[i].clone();
                        let handle = task::spawn(async move {
                            let mut brain_lock = brain.lock().unwrap();
                            brain_lock.tick(&player, &high_level_state, episode)
                        });
                        action_handles.push(handle);
                    }
                }

                let actions_results: Vec<_> = futures::future::join_all(action_handles).await;

                for (i, result) in actions_results.into_iter().enumerate() {
                    if self.players[i].health > 0 {
                        match result {
                            Ok(Ok(action)) => {
                                let state_before_action = self.get_high_level_state(i);
                                let goal_before_action = self.brains[i].lock().unwrap().current_goal.clone();

                                self._perform_action(i, &action, episode);

                                if let Some(goal) = goal_before_action {
                                    if self.brains[i].lock().unwrap().is_goal_complete(&self.players[i], &goal) {
                                        let reward = match goal {
                                            super::brain::Goal::GatherResource(_) => 50.0,
                                            super::brain::Goal::CraftItem(_) => 200.0,
                                            _ => 10.0,
                                        };
                                        let next_state = self.get_high_level_state(i);
                                        self.brains[i].lock().unwrap().update_goal_q_table(&state_before_action, &goal, reward, &next_state)?;
                                    }
                                }
                            },
                            Ok(Err(e)) => return Err(e.into()),
                            Err(e) => return Err(SimulationError::TaskJoinError(e.to_string())),
                        }
                    }
                }
            }

            let mut brain_lock = self.brains[0].lock().unwrap();
            if brain_lock.epsilon > MIN_EPSILON {
                brain_lock.epsilon *= EPSILON_DECAY;
            }

            if (episode + 1) % 200 == 0 {
                println!("Episode {}/{} | P1 Epsilon: {:.3}", brain_lock.epsilon, episode + 1, EPISODES);
                self._display_mental_map(0);
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
                        tile.tile_type = tile.original_tile_type;
                        tile.remaining_resources = Some(5);
                        tile.depletion_episode = None;
                    }
                }
            }
        }
    }

    fn _display_mental_map(&self, player_index: usize) {
        println!("--- Player {} Mental Map ---", player_index);
        let brain = self.brains[player_index].lock().unwrap();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                match &brain.mental_map[y as usize][x as usize] {
                    Some(memory_tile) => print!("{} ", memory_tile.tile.tile_type),
                    None => print!("? "),
                }
            }
            println!();
        }
        println!("--------------------------");
    }

    fn _handle_equip_action(&mut self, player_index: usize, item: &str) -> f64 {
        let player = &mut self.players[player_index];
        if player.get_total_quantity(item) > 0 {
            player.held_item = Some(item.to_string());
            2.0
        } else {
            -2.0
        }
    }

    fn _handle_craft_action(&mut self, player_index: usize, item: &str) -> f64 {
        let required_resources = self.recipe_manager.get_required_resources(item, 1);

        if !required_resources.is_empty() && required_resources.get(item).is_none() {
            let player = &mut self.players[player_index];
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
    }

    fn _handle_move_action(&mut self, player_index: usize, direction: &Direction) -> f64 {
        let player = &mut self.players[player_index];
        let direction_str = match direction {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Left => "left",
            Direction::Right => "right",
        };
        if player.move_player(direction_str, &self.map) {
            let (new_px, new_py) = (player.x, player.y);
            let current_tile = &self.map.grid[new_py as usize][new_px as usize];
            if current_tile.tile_type == 'M' { -2.0 }
            else if "RUIT".contains(current_tile.tile_type) { 1.0 }
            else { 0.0 }
        } else { -5.0 }
    }

    fn _handle_gather_action(&mut self, player_index: usize, px: u32, py: u32, episode: u32) -> f64 {
        let tile = &mut self.map.grid[py as usize][px as usize];
        if tile.remaining_resources.is_none() || tile.remaining_resources == Some(0) {
            return -2.0; // Nothing to gather
        }

        let player = &mut self.players[player_index];
        let held_item_name = player.held_item.as_deref();
        let held_item = held_item_name.and_then(|name| self.item_registry.get_item(name));

        if let Some(item) = held_item {
            if let Some(properties) = &item.properties {
                let efficiency = properties.get("efficiency").cloned().unwrap_or(1.0) as u32;
                let required_tool = match tile.tile_type {
                    'T' => "stone_axe",
                    'R' => "stone_pickaxe",
                    'U' => "stone_pickaxe",
                    'I' => "metal_pickaxe",
                    _ => "",
                };

                if item.name == required_tool {
                    let resource = match tile.tile_type {
                        'T' => "wood",
                        'R' => "stone",
                        'U' => "sulfur",
                        'I' => "iron_ore",
                        _ => "",
                    };

                    if player.add_item(resource, efficiency, None, &self.item_registry) {
                        if let Some(res) = &mut tile.remaining_resources {
                            *res -= 1;
                            if *res == 0 {
                                tile.tile_type = '.';
                                tile.depletion_episode = Some(episode);
                            }
                        }

                        let player = &mut self.players[player_index];
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
                    } else { -15.0 }
                } else { -10.0 }
            } else { -10.0 }
        } else { -10.0 }
    }

    fn _handle_place_furnace_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        let player = &mut self.players[player_index];
        if player.get_total_quantity("furnace") > 0 && self.map.grid[py as usize][px as usize].tile_type == '.' {
            let mut recipe = HashMap::new(); recipe.insert("furnace".to_string(), 1);
            player.remove_resources(&recipe);
            self.map.grid[py as usize][px as usize].tile_type = 'F';
            40.0
        } else { -5.0 }
    }

    fn _handle_place_door_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        let player = &mut self.players[player_index];
        if player.get_total_quantity("door") > 0 && self.map.grid[py as usize][px as usize].tile_type == 'O' {
            let mut recipe = HashMap::new(); recipe.insert("door".to_string(), 1);
            player.remove_resources(&recipe);
            self.map.grid[py as usize][px as usize].tile_type = 'D';
            40.0
        } else { -5.0 }
    }

    fn _handle_smelt_iron_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        let mut recipe = HashMap::new();
        recipe.insert("iron_ore".to_string(), 1);
        recipe.insert("wood".to_string(), 1);

        if self._is_adjacent_to(px, py, 'F') {
            let player = &mut self.players[player_index];
            if player.has_resources(&recipe) {
                if player.remove_resources(&recipe) {
                    player.add_item("iron_bars", 1, None, &self.item_registry);
                    60.0
                } else { -15.0 }
            } else { -12.0 }
        } else { -12.0 }
    }

    fn _handle_build_action(&mut self, player_index: usize, structure: &str, px: u32, py: u32) -> f64 {
        let player = &mut self.players[player_index];
        if player.get_total_quantity(structure) > 0 {
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
    }

    fn _handle_attach_lock_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        let player = &mut self.players[player_index];
        if !player.has_lock() {
            return -10.0;
        }

        if let Some((door_x, door_y)) = self._find_adjacent_tile(px, py, 'D') {
            let door_tile = &mut self.map.grid[door_y as usize][door_x as usize];
            if door_tile.lock_id.is_none() {
                let player = &mut self.players[player_index];
                if let Some(lock_id) = player.find_and_remove_lock() {
                    door_tile.tile_type = 'L';
                    door_tile.lock_id = Some(lock_id);
                    return 50.0;
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
                let player = &self.players[player_index];
                if player.has_key(lock_id) {
                    self.map.grid[door_y as usize][door_x as usize].tile_type = 'd';
                    return 20.0;
                } else {
                    return -15.0;
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
        let attacker_id = self.players[player_index].id;
        let held_item_name = self.players[player_index].held_item.clone();
        let damage = if let Some(item_name) = &held_item_name {
            self.item_registry.get_item(item_name).and_then(|item| item.properties.as_ref()).and_then(|p| p.get("damage")).cloned().unwrap_or(1.0)
        } else { 1.0 };

        // Try to attack a building first
        let (px, py) = (self.players[player_index].x, self.players[player_index].y);
        for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let nx = (px as i32 + dx) as u32;
            let ny = (py as i32 + dy) as u32;
            if nx < self.map.width && ny < self.map.height {
                let tile = &mut self.map.grid[ny as usize][nx as usize];
                if "B#DO".contains(tile.tile_type) {
                    if let Some(health) = &mut tile.health {
                        *health -= damage;
                        if *health <= 0.0 {
                            let was_foundation = tile.tile_type == 'B';
                            tile.tile_type = '.';
                            tile.health = None;
                            if was_foundation {
                                self._handle_structural_collapse(nx, ny);
                            }
                        }
                        return 20.0;
                    }
                }
            }
        }

        // If no building, try to attack a player
        if let Some(other_player_index) = self._find_adjacent_player(player_index) {
            let (players_slice1, players_slice2) = self.players.split_at_mut(std::cmp::max(player_index, other_player_index));
            let (attacker, victim) = if player_index < other_player_index {
                (&mut players_slice1[player_index], &mut players_slice2[0])
            } else {
                (&mut players_slice2[0], &mut players_slice1[other_player_index])
            };

            victim.health -= damage as i32;

            let victim_brain = Arc::clone(&self.brains[other_player_index]);
            let mut victim_brain_lock = victim_brain.lock().unwrap();
            victim_brain_lock.record_attack_from(attacker_id);

            if let Some(item_name) = held_item_name {
                if let Some(slot) = attacker.inventory.iter_mut().find(|s| s.is_some() && s.as_ref().unwrap().item == item_name) {
                    if let Some(s) = slot {
                        if let Some(durability) = &mut s.durability {
                            *durability -= 1.0;
                            if *durability <= 0.0 {
                                *slot = None;
                            }
                        }
                    }
                }
            }

            if victim.health <= 0 {
                100.0
            } else {
                10.0
            }
        } else {
            -1.0
        }
    }

    pub fn _perform_action(&mut self, player_index: usize, action: &Action, episode: u32) -> f64 {
        let (px, py) = (self.players[player_index].x, self.players[player_index].y);

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
        }
    }
}

fn get_random_samples<T: Clone>(population: &[T], k: usize, rng: &mut impl Rng) -> Vec<T> {
    let mut samples = Vec::new();
    let mut indices: Vec<usize> = (0..population.len()).collect();
    for _ in 0..k {
        if indices.is_empty() { break; }
        let i = rng.gen_range(0..indices.len());
        let selected_index = indices.swap_remove(i);
        samples.push(population[selected_index].clone());
    }
    samples
}
