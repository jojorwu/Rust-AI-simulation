use std::collections::HashMap;
use super::map::Map;
use super::player::Player;
use super::brain::{Brain, HighLevelState};
use super::recipes::RecipeManager;
use super::errors::SimulationError;
use super::actions::{Action, Direction, get_all_actions};
use super::item::ItemRegistry;
use super::ecs::World;
use super::components::{Position, Velocity};
use super::systems::movement_system;
use std::any::Any;
use std::sync::{Arc, Mutex};
use tokio::task;

use rand::Rng;

use super::config::*;


pub struct Game {
    pub map: Map,
    pub world: World,
    pub item_registry: ItemRegistry,
    pub recipe_manager: Arc<RecipeManager>,
    next_instance_id: u32,
}

impl Game {
    pub fn new() -> Self {
        let map = Map::new(WIDTH, HEIGHT).expect("Failed to create map");
        let item_registry = ItemRegistry::new("items.json");
        let recipe_manager = Arc::new(RecipeManager::new("recipes.json"));

        let mut world = World::new();
        let actions = get_all_actions();

        for i in 0..NUM_PLAYERS {
            let player = world.create_entity();
            world.add_component(player, Player::new(i as u32));
            world.add_component(player, Position { x: 0, y: 0 });
        }

        let animal = world.create_entity();
        world.add_component(animal, Animal {
            id: 100,
            health: 50,
            species: "deer".to_string(),
        });
        world.add_component(animal, Position { x: 5, y: 5 });

        Game {
            map,
            world,
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

    // This function needs to be re-thought with the ECS architecture
    // For now, it will just return None

    fn _find_and_set_valid_start_positions(&mut self) {
        let mut rng = rand::thread_rng();
        let mut occupied_positions = std::collections::HashSet::new();

        let plains_biome = self.map.biomes.iter().find(|b| b.name == "plains");
        let plains_tile_type = plains_biome.map_or('.', |b| b.tile_type);

        for entity in 0..self.world.entities.len() {
            loop {
                let x = rng.gen_range(0..self.map.width);
                let y = rng.gen_range(0..self.map.height);
                if self.map.grid[y as usize][x as usize].tile_type == plains_tile_type && !occupied_positions.contains(&(x, y)) {
                    if let Some(pos) = self.world.get_component_mut::<Position>(entity) {
                        pos.x = x;
                        pos.y = y;
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
        // self.map.display(&self.world); // This needs to be updated

        for episode in 0..EPISODES {
            self._respawn_resources(episode);
            // player.reset() will need to be re-implemented
            self._find_and_set_valid_start_positions();

            for _step in 0..MAX_STEPS_PER_EPISODE {
                // For now, we will just give the first player a random move action
                let mut rng = rand::thread_rng();
                let direction = match rng.gen_range(0..4) {
                    0 => Direction::Up,
                    1 => Direction::Down,
                    2 => Direction::Left,
                    _ => Direction::Right,
                };
                self._perform_action(0, &Action::Move(direction), episode);
                movement_system(&mut self.world);
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





    fn _handle_gather_action(&mut self, player_entity: Entity, px: u32, py: u32, episode: u32) -> f64 {
        let (tile_type, biome_name) = {
            let tile = &self.map.grid[py as usize][px as usize];
            (tile.tile_type, tile.biome.clone())
        };

        if self.map.grid[py as usize][px as usize].remaining_resources.is_none() || self.map.grid[py as usize][px as usize].remaining_resources == Some(0) {
            return -2.0; // Nothing to gather
        }

        if let Some(player) = self.world.get_component_mut::<Player>(player_entity) {
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

    fn _handle_place_furnace_action(&mut self, player_entity: Entity, px: u32, py: u32) -> f64 {
        if let Some(player) = self.world.get_component_mut::<Player>(player_entity) {
            if player.get_total_quantity("furnace") > 0 && self.map.grid[py as usize][px as usize].tile_type == '.' {
                // Check for other entities
                // This needs to be re-thought with the ECS architecture
                let mut recipe = HashMap::new(); recipe.insert("furnace".to_string(), 1);
                player.remove_resources(&recipe);
                self.map.grid[py as usize][px as usize].tile_type = 'F';
                40.0
            } else { -5.0 }
        } else { -1.0 }
    }

    fn _handle_place_door_action(&mut self, player_entity: Entity, px: u32, py: u32) -> f64 {
        if let Some(player) = self.world.get_component_mut::<Player>(player_entity) {
            if player.get_total_quantity("door") > 0 && self.map.grid[py as usize][px as usize].tile_type == 'O' {
                // Check for other entities
                // This needs to be re-thought with the ECS architecture
                let mut recipe = HashMap::new(); recipe.insert("door".to_string(), 1);
                player.remove_resources(&recipe);
                self.map.grid[py as usize][px as usize].tile_type = 'D';
                40.0
            } else { -5.0 }
        } else { -1.0 }
    }

    fn _handle_smelt_iron_action(&mut self, player_entity: Entity, px: u32, py: u32) -> f64 {
        let mut recipe = HashMap::new();
        recipe.insert("iron_ore".to_string(), 1);
        recipe.insert("wood".to_string(), 1);

        if self._is_adjacent_to(px, py, 'F') {
            if let Some(player) = self.world.get_component_mut::<Player>(player_entity) {
                if player.has_resources(&recipe) {
                    if player.remove_resources(&recipe) {
                        player.add_item("iron_bars", 1, None, &self.item_registry);
                        60.0
                    } else { -15.0 }
                } else { -12.0 }
            } else { -1.0 }
        } else { -12.0 }
    }

    fn _handle_build_action(&mut self, player_entity: Entity, structure: &str, px: u32, py: u32) -> f64 {
        if let Some(player) = self.world.get_component_mut::<Player>(player_entity) {
            if player.get_total_quantity(structure) > 0 {
                // Check for other entities
                // This needs to be re-thought with the ECS architecture

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

    fn _handle_attach_lock_action(&mut self, player_entity: Entity, px: u32, py: u32) -> f64 {
        let has_lock = if let Some(player) = self.world.get_component::<Player>(player_entity) {
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
                if let Some(player) = self.world.get_component_mut::<Player>(player_entity) {
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

    fn _handle_open_door_action(&mut self, player_entity: Entity, px: u32, py: u32) -> f64 {
        if let Some((door_x, door_y)) = self._find_adjacent_tile(px, py, 'D') {
            self.map.grid[door_y as usize][door_x as usize].tile_type = 'd';
            return 10.0;
        }

        if let Some((door_x, door_y)) = self._find_adjacent_tile(px, py, 'L') {
            let door_tile = &self.map.grid[door_y as usize][door_x as usize];
            if let Some(lock_id) = door_tile.lock_id {
                if let Some(player) = self.world.get_component::<Player>(player_entity) {
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

    fn _handle_attack_action(&mut self, player_entity: Entity) -> f64 {
        // This needs to be re-thought with the ECS architecture
        -1.0
    }

    fn _handle_pickup_action(&mut self, player_entity: Entity) -> f64 {
        // This needs to be re-thought with the ECS architecture
        -1.0
    }

    pub fn _perform_action(&mut self, player_entity: Entity, action: &Action, episode: u32) -> f64 {
        let (px, py) = self.world.get_component::<Position>(player_entity).map_or((0, 0), |p| (p.x, p.y));

        match action {
            Action::Move(direction) => {
                let (dx, dy) = match direction {
                    Direction::Up => (0, -1),
                    Direction::Down => (0, 1),
                    Direction::Left => (-1, 0),
                    Direction::Right => (1, 0),
                };
                self.world.add_component(player_entity, Velocity { dx, dy });
                0.0
            }
            Action::Gather => self._handle_gather_action(player_entity, px, py, episode),
            Action::Craft(item) => self._handle_craft_action(player_entity, item),
            Action::Equip(item) => self._handle_equip_action(player_entity, item),
            Action::Place(item) => {
                if item == "furnace" {
                    self._handle_place_furnace_action(player_entity, px, py)
                } else if item == "door" {
                    self._handle_place_door_action(player_entity, px, py)
                } else {
                    -0.1
                }
            },
            Action::Smelt => self._handle_smelt_iron_action(player_entity, px, py),
            Action::Build(structure) => self._handle_build_action(player_entity, structure, px, py),
            Action::Open => self._handle_open_door_action(player_entity, px, py),
            Action::Close => self._handle_close_door_action(player_entity, px, py),
            Action::AttachLock => self._handle_attach_lock_action(player_entity, px, py),
            Action::Attack => self._handle_attack_action(player_entity),
            Action::Pickup => self._handle_pickup_action(player_entity),
            _ => -1.0,
        }
    }
}

