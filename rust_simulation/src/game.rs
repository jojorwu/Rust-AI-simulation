use std::collections::HashMap;
use super::map::{Map, Tile};
use super::player::Player;
use super::brain::Brain;
use super::state::StateKey;
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
    pub recipe_manager: RecipeManager,
    next_instance_id: u32,
}

impl Game {
    pub fn new() -> Self {
        let map = Map::new(WIDTH, HEIGHT);
        let item_registry = ItemRegistry::new("items.json");
        let recipe_manager = RecipeManager::new("recipes.json");

        let mut players = Vec::new();
        let mut brains = Vec::new();
        let actions = get_all_actions();

        for _ in 0..NUM_PLAYERS {
            players.push(Player::new(0, 0));
            brains.push(Arc::new(Mutex::new(Brain::new(actions.clone(), LEARNING_RATE, DISCOUNT_FACTOR, INITIAL_EPSILON))));
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

    fn get_state(&mut self, player_index: usize) -> StateKey {
        let player = &self.players[player_index];
        let mut local_view = Vec::new();
        let view_radius = 1;

        let mut brain_lock = self.brains[player_index].lock().unwrap();

        for dy in -view_radius..=view_radius {
            for dx in -view_radius..=view_radius {
                let nx = player.x as i32 + dx;
                let ny = player.y as i32 + dy;

                if nx >= 0 && nx < self.map.width as i32 && ny >= 0 && ny < self.map.height as i32 {
                    let tile = self.map.grid[ny as usize][nx as usize].clone();
                    local_view.push(tile.clone());
                    brain_lock.mental_map[ny as usize][nx as usize] = Some(tile);
                } else {
                    local_view.push(Tile::new('X'));
                }
            }
        }

        StateKey {
            local_view,
            inventory: player.inventory.clone(),
            held_item: player.held_item.clone(),
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
                        let state = self.get_state(i);
                        let brain = Arc::clone(&self.brains[i]);
                        let player_pos = (self.players[i].x, self.players[i].y);
                        let handle = task::spawn(async move {
                            let mut brain_lock = brain.lock().unwrap();
                            brain_lock.choose_action(&state, player_pos)
                        });
                        action_handles.push(handle);
                    }
                }

                let actions_results: Vec<_> = futures::future::join_all(action_handles).await;

                for (i, result) in actions_results.into_iter().enumerate() {
                    if self.players[i].health > 0 {
                         match result {
                            Ok(Ok(action)) => {
                                let reward = self._perform_action(i, &action, episode);
                                let state_before_action = self.get_state(i);
                                let next_state = self.get_state(i);
                                self.brains[i].lock().unwrap().update_q_table(&state_before_action, &action, reward, &next_state)?;
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
                    Some(tile) => print!("{} ", tile.tile_type),
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
        let held = player.held_item.as_deref();
        let tool_map: HashMap<char, (&str, &str, f64)> = [
            ('T', ("stone_axe", "wood", 20.0)),
            ('R', ("stone_pickaxe", "stone", 20.0)),
            ('U', ("stone_pickaxe", "sulfur", 30.0)),
            ('I', ("metal_pickaxe", "iron_ore", 40.0)),
        ].iter().cloned().collect();

        if let Some((required_tool, resource, reward_val)) = tool_map.get(&tile.tile_type) {
            if held == Some(*required_tool) {
                if player.add_item(resource, 3, None, &self.item_registry) {
                    if let Some(res) = &mut tile.remaining_resources {
                        *res -= 1;
                        if *res == 0 {
                            tile.tile_type = '.';
                            tile.depletion_episode = Some(episode);
                        }
                    }
                    *reward_val
                } else { -15.0 }
            } else { -10.0 }
        } else { -2.0 }
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
        let _player = &mut self.players[player_index];
        let current_tile = &mut self.map.grid[py as usize][px as usize];

        match structure {
            "foundation" => {
                if current_tile.tile_type == '.' {
                    current_tile.tile_type = 'B';
                    30.0
                } else { -5.0 }
            }
            "wall" => {
                if current_tile.tile_type == 'B' {
                    current_tile.tile_type = '#';
                    30.0
                } else { -5.0 }
            }
            "doorway" => {
                if current_tile.tile_type == 'B' {
                    current_tile.tile_type = 'O';
                    30.0
                } else { -5.0 }
            }
            _ => -0.1,
        }
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

    fn _handle_attack_action(&mut self, player_index: usize) -> f64 {
        if let Some(other_player_index) = self._find_adjacent_player(player_index) {
            self.players[other_player_index].health -= 10;
            if self.players[other_player_index].health <= 0 {
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
