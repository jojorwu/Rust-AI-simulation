use std::collections::HashMap;
use super::map::Map;
use super::player::Player;
use super::brain::Brain;
use super::state::StateKey;
use super::recipes;
use super::errors::SimulationError;

use rand::Rng;

use super::config::*;


pub struct Game {
    pub map: Map,
    pub players: Vec<Player>,
    pub brains: Vec<Brain>,
    pub cycle_successes: u32,
    pub last_cycle_performance: f64,
    pub current_cycle_episodes: u32,
}

impl Game {
    pub fn new() -> Self {
        let map = Map::new(WIDTH, HEIGHT);

        let mut players = Vec::new();
        let mut brains = Vec::new();
        let actions = vec![
            "up".to_string(), "down".to_string(), "left".to_string(), "right".to_string(), "gather".to_string(),
            "craft_stone_axe".to_string(), "craft_stone_pickaxe".to_string(), "craft_furnace".to_string(), "craft_metal_pickaxe".to_string(),
            "equip_stone_axe".to_string(), "equip_stone_pickaxe".to_string(), "equip_metal_pickaxe".to_string(),
            "place_furnace".to_string(), "smelt_iron".to_string()
        ];

        for i in 0..NUM_PLAYERS {
            players.push(Player::new(i, 0, 0));
            brains.push(Brain::new(actions.clone(), LEARNING_RATE, DISCOUNT_FACTOR, INITIAL_EPSILON));
        }

        Game {
            map,
            players,
            brains,
            cycle_successes: 0,
            last_cycle_performance: 0.0,
            current_cycle_episodes: 0,
        }
    }

    fn get_state(&self, player_index: usize) -> StateKey {
        let player = &self.players[player_index];
        let mut local_view = Vec::new();
        let view_radius = 1; // 3x3 view

        for dy in -view_radius..=view_radius {
            for dx in -view_radius..=view_radius {
                let nx = player.x as i32 + dx;
                let ny = player.y as i32 + dy;

                if nx >= 0 && nx < self.map.width as i32 && ny >= 0 && ny < self.map.height as i32 {
                    local_view.push(self.map.grid[ny as usize][nx as usize]);
                } else {
                    local_view.push('X'); // 'X' for out of bounds
                }
            }
        }

        StateKey {
            local_view,
            inventory: player.inventory.clone(),
            held_item: player.held_item.clone(),
        }
    }

    fn _is_adjacent_to(&self, player_index: usize, tile_type: char) -> bool {
        let player = &self.players[player_index];
        let px = player.x;
        let py = player.y;
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
        let mut original_map_grid = self.map.grid.clone();
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
            }
        }

        println!("--- Training Finished ---");
        Ok(())
    }

    pub fn _perform_action(&mut self, player_index: usize, action: &str) -> f64 {
        let mut reward = -0.1;

        // --- Get Recipes ---
        let recipes = recipes::get_recipes();

        // --- Actions ---
        let player = &mut self.players[player_index];

        if action.starts_with("equip_") {
            let item = &action[6..];
            if player.get_total_quantity(item) > 0 {
                player.held_item = Some(item.to_string());
                reward = 2.0;
            } else {
                reward = -2.0;
            }
        } else if action.starts_with("craft_") {
            let item = &action[6..];
            if let Some(recipe) = recipes.get(item) {
                if player.has_resources(recipe) {
                    if player.remove_resources(recipe) {
                        if player.add_item(item, 1) {
                             reward = 50.0;
                        } else { reward = -15.0; } // Inventory full
                    } else { reward = -15.0; } // Should not happen
                } else { reward = -10.0; }
            } else { reward = -1.0; }
        } else {
            match action {
                "up" | "down" | "left" | "right" => {
                    if player.move_player(action, &self.map) {
                        let current_tile = self.map.grid[player.y as usize][player.x as usize];
                        if current_tile == 'M' { reward = -2.0; }
                        else if "RUIT".contains(current_tile) { reward = 1.0; }
                    } else { reward = -5.0; }
                },
                "gather" => {
                    let tile = self.map.grid[player.y as usize][player.x as usize];
                    let held = player.held_item.as_deref();
                    let tool_map: HashMap<char, (&str, &str, f64)> = [
                        ('T', ("stone_axe", "wood", 20.0)),
                        ('R', ("stone_pickaxe", "stone", 20.0)),
                        ('U', ("stone_pickaxe", "sulfur", 30.0)),
                        ('I', ("metal_pickaxe", "iron_ore", 40.0)),
                    ].iter().cloned().collect();

                    if let Some((required_tool, resource, reward_val)) = tool_map.get(&tile) {
                        if held == Some(*required_tool) {
                            if player.add_item(resource, 1) {
                                self.map.grid[player.y as usize][player.x as usize] = '.';
                                reward = *reward_val;
                            } else { reward = -15.0; }
                        } else { reward = -10.0; }
                    } else { reward = -2.0; }
                },
                "place_furnace" => {
                    if player.get_total_quantity("furnace") > 0 && self.map.grid[player.y as usize][player.x as usize] == '.' {
                        let mut recipe = HashMap::new(); recipe.insert("furnace".to_string(), 1);
                        player.remove_resources(&recipe);
                        self.map.grid[player.y as usize][player.x as usize] = 'F';
                        reward = 40.0;
                    } else { reward = -5.0; }
                },
                "smelt_iron" => {
                    let mut recipe = HashMap::new(); recipe.insert("iron_ore".to_string(), 1); recipe.insert("wood".to_string(), 1);
                    if self._is_adjacent_to(player_index, 'F') && player.has_resources(&recipe) {
                        if player.remove_resources(&recipe) {
                            player.add_item("iron_bars", 1);
                            reward = 60.0;
                        } else { reward = -15.0; }
                    } else { reward = -12.0; }
                },
                _ => (), // Unknown action
            }
        }
        reward
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
