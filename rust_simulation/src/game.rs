use std::collections::HashMap;
use super::map::Map;
use super::player::Player;
use super::brain::Brain;
use super::state::StateKey;
use super::recipes;
use super::errors::SimulationError;
use super::actions::{Action, Direction, get_all_actions};
use super::item::ItemRegistry;

use rand::Rng;

use super::config::*;


pub struct Game {
    pub map: Map,
    pub players: Vec<Player>,
    pub brains: Vec<Brain>,
    pub item_registry: ItemRegistry,
}

impl Game {
    pub fn new() -> Self {
        let map = Map::new(WIDTH, HEIGHT);
        let item_registry = ItemRegistry::new("items.json");

        let mut players = Vec::new();
        let mut brains = Vec::new();
        let actions = get_all_actions();

        for _ in 0..NUM_PLAYERS {
            players.push(Player::new(0, 0));
            brains.push(Brain::new(actions.clone(), LEARNING_RATE, DISCOUNT_FACTOR, INITIAL_EPSILON));
        }

        Game {
            map,
            players,
            brains,
            item_registry,
        }
    }

    fn get_state(&mut self, player_index: usize) -> StateKey {
        let player = &self.players[player_index];
        let mut local_view = Vec::new();
        let view_radius = 1; // 3x3 view

        for dy in -view_radius..=view_radius {
            for dx in -view_radius..=view_radius {
                let nx = player.x as i32 + dx;
                let ny = player.y as i32 + dy;

                if nx >= 0 && nx < self.map.width as i32 && ny >= 0 && ny < self.map.height as i32 {
                    let tile = self.map.grid[ny as usize][nx as usize];
                    local_view.push(tile);
                    // Update mental map
                    self.brains[player_index].mental_map[ny as usize][nx as usize] = Some(tile);
                } else {
                    local_view.push('X'); // 'X' for out of bounds
                }
            }
        }

        StateKey {
            local_view,
            inventory: player.inventory.clone(),
            held_item: player.held_item.clone(),
            mental_map: self.brains[player_index].mental_map.clone(),
        }
    }

    fn _is_adjacent_to(&self, px: u32, py: u32, tile_type: char) -> bool {
        for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let nx = (px as i32 + dx) as u32;
            let ny = (py as i32 + dy) as u32;
            if nx < self.map.width && ny < self.map.height {
                if self.map.grid[ny as usize][nx as usize] == tile_type {
                    return true;
                }
            }
        }
        false
    }

    fn _find_and_set_valid_start_positions(&mut self) {
        let mut rng = rand::thread_rng();
        let mut occupied_positions = std::collections::HashSet::new();

        for player in &mut self.players {
            loop {
                let x = rng.gen_range(0..self.map.width);
                let y = rng.gen_range(0..self.map.height);
                if self.map.grid[y as usize][x as usize] == '.' && !occupied_positions.contains(&(x, y)) {
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
                match self.map.grid[y as usize][x as usize] {
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

    pub fn run(&mut self) -> Result<(), SimulationError> {
        println!("--- Starting Rust Training Simulation ---");
        self.setup_new_map();
        let original_map_grid = self.map.grid.clone();
        self._find_and_set_valid_start_positions();

        println!("Initial Map:");
        self.map.display(&self.players);

        for episode in 0..EPISODES {
            // ... (wipe logic can be added here later) ...

            self.map.grid = original_map_grid.clone();
            for player in &mut self.players {
                player.reset();
            }
            self._find_and_set_valid_start_positions();

            for _step in 0..MAX_STEPS_PER_EPISODE {
                for i in 0..self.players.len() {
                    let state = self.get_state(i);
                    let action = self.brains[i].choose_action(&state)?;
                    let reward = self._perform_action(i, &action);
                    let next_state = self.get_state(i);
                    self.brains[i].update_q_table(&state, &action, reward, &next_state)?;
                }
            }

            if self.brains[0].epsilon > MIN_EPSILON {
                self.brains[0].epsilon *= EPSILON_DECAY;
            }

            if (episode + 1) % 200 == 0 {
                println!("Episode {}/{} | P1 Epsilon: {:.3}", episode + 1, EPISODES, self.brains[0].epsilon);
                self._display_mental_map(0);
            }
        }

        println!("--- Training Finished ---");
        Ok(())
    }

    fn _display_mental_map(&self, player_index: usize) {
        println!("--- Player {} Mental Map ---", player_index);
        let brain = &self.brains[player_index];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                match brain.mental_map[y as usize][x as usize] {
                    Some(tile) => print!("{} ", tile),
                    None => print!("? "),
                }
            }
            println!();
        }
        println!("--------------------------");
    }

    // --- Action Handler Methods ---

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
        let player = &mut self.players[player_index];
        let recipes = recipes::get_recipes();
        if let Some(recipe) = recipes.get(item) {
            if player.has_resources(recipe) {
                if player.remove_resources(recipe) {
                    if player.add_item(item, 1, &self.item_registry) {
                        50.0
                    } else { -15.0 }
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
            let current_tile = self.map.grid[new_py as usize][new_px as usize];
            if current_tile == 'M' { -2.0 }
            else if "RUIT".contains(current_tile) { 1.0 }
            else { 0.0 }
        } else { -5.0 }
    }

    fn _handle_gather_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        let tile = self.map.grid[py as usize][px as usize];
        let player = &mut self.players[player_index];
        let held = player.held_item.as_deref();
        let tool_map: HashMap<char, (&str, &str, f64)> = [
            ('T', ("stone_axe", "wood", 20.0)),
            ('R', ("stone_pickaxe", "stone", 20.0)),
            ('U', ("stone_pickaxe", "sulfur", 30.0)),
            ('I', ("metal_pickaxe", "iron_ore", 40.0)),
        ].iter().cloned().collect();

        if let Some((required_tool, resource, reward_val)) = tool_map.get(&tile) {
            if held == Some(*required_tool) {
                if player.add_item(resource, 1, &self.item_registry) {
                    self.map.grid[py as usize][px as usize] = '.';
                    *reward_val
                } else { -15.0 }
            } else { -10.0 }
        } else { -2.0 }
    }

    fn _handle_place_furnace_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        let player = &mut self.players[player_index];
        if player.get_total_quantity("furnace") > 0 && self.map.grid[py as usize][px as usize] == '.' {
            let mut recipe = HashMap::new(); recipe.insert("furnace".to_string(), 1);
            player.remove_resources(&recipe);
            self.map.grid[py as usize][px as usize] = 'F';
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
                    player.add_item("iron_bars", 1, &self.item_registry);
                    60.0
                } else { -15.0 }
            } else { -12.0 }
        } else { -12.0 }
    }

    fn _handle_build_foundation_action(&mut self, player_index: usize, px: u32, py: u32) -> f64 {
        let _player = &mut self.players[player_index];
        if self.map.grid[py as usize][px as usize] == '.' {
            self.map.grid[py as usize][px as usize] = 'B';
            30.0
        } else {
            -5.0
        }
    }

    pub fn _perform_action(&mut self, player_index: usize, action: &Action) -> f64 {
        let (px, py) = (self.players[player_index].x, self.players[player_index].y);

        match action {
            Action::Move(direction) => self._handle_move_action(player_index, direction),
            Action::Gather => self._handle_gather_action(player_index, px, py),
            Action::Craft(item) => self._handle_craft_action(player_index, item),
            Action::Equip(item) => self._handle_equip_action(player_index, item),
            Action::Place(item) => {
                if item == "furnace" {
                    self._handle_place_furnace_action(player_index, px, py)
                } else {
                    -0.1
                }
            },
            Action::Smelt => self._handle_smelt_iron_action(player_index, px, py),
            Action::Build(structure) => {
                if structure == "foundation" {
                    self._handle_build_foundation_action(player_index, px, py)
                } else {
                    -0.1
                }
            },
        }
    }
}

// Helper function for random sampling without replacement
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
