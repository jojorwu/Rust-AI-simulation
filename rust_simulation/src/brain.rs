use std::collections::HashMap;
use rand::Rng;
use super::state::StateKey;
use super::errors::SimulationError;
use super::config::{WIDTH, HEIGHT};
use super::actions::{Action, Direction};
use super::map::Tile;
use super::pathfinding;

#[derive(Debug, Clone, PartialEq)]
pub enum Goal {
    GatherResource { resource_char: char, resource_name: String },
    FleeFromHostile { hostile_player_id: u32 },
}

#[derive(Debug, Clone)]
pub struct MemoryTile {
    pub tile: Tile,
    pub last_seen_episode: u32,
    pub resource_richness: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipStatus {
    Neutral,
    Hostile,
}

#[derive(Debug, Clone)]
pub struct PlayerMemory {
    pub last_seen_location: Option<(u32, u32)>,
    pub relationship: RelationshipStatus,
}

pub struct Brain {
    pub actions: Vec<Action>,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub epsilon: f64,
    pub q_table: HashMap<String, HashMap<Action, f64>>,
    pub mental_map: Vec<Vec<Option<MemoryTile>>>,
    pub player_memories: HashMap<u32, PlayerMemory>,
    pub current_goal: Option<Goal>,
    pub current_path: Option<Vec<(u32, u32)>>,
}

impl Brain {
    pub fn new(actions: Vec<Action>, learning_rate: f64, discount_factor: f64, epsilon: f64) -> Self {
        Brain {
            actions,
            learning_rate,
            discount_factor,
            epsilon,
            q_table: HashMap::new(),
            mental_map: vec![vec![None; WIDTH as usize]; HEIGHT as usize],
            player_memories: HashMap::new(),
            current_goal: None,
            current_path: None,
        }
    }

    fn find_best_resource_spot(&self, player_pos: (u32, u32), resource_char: char) -> Option<(u32, u32)> {
        let mut best_score = f64::MIN;
        let mut best_pos = None;

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if let Some(memory_tile) = &self.mental_map[y as usize][x as usize] {
                    if memory_tile.tile.tile_type == resource_char {
                        let dist = ((player_pos.0 as f64 - x as f64).powi(2) + (player_pos.1 as f64 - y as f64).powi(2)).sqrt();
                        // Simple score: richness divided by distance. Higher is better.
                        let score = memory_tile.resource_richness as f64 / (dist + 1.0);
                        if score > best_score {
                            best_score = score;
                            best_pos = Some((x, y));
                        }
                    }
                }
            }
        }
        best_pos
    }

    pub fn choose_action(&mut self, state: &StateKey, player_pos: (u32, u32)) -> Result<Action, SimulationError> {
        // 1. Check for immediate threats and react
        for (id, memory) in &self.player_memories {
            if memory.relationship == RelationshipStatus::Hostile {
                if let Some(loc) = memory.last_seen_location {
                    let dist = ((player_pos.0 as f64 - loc.0 as f64).powi(2) + (player_pos.1 as f64 - loc.1 as f64).powi(2)).sqrt();
                    if dist < 5.0 {
                        self.current_goal = Some(Goal::FleeFromHostile { hostile_player_id: *id });
                        self.current_path = None; // Cancel current path

                        let dx = player_pos.0 as i32 - loc.0 as i32;
                        let dy = player_pos.1 as i32 - loc.1 as i32;

                        // Move away from the hostile player
                        return Ok(Action::Move(if dx.abs() > dy.abs() {
                            if dx > 0 { Direction::Right } else { Direction::Left }
                        } else {
                            if dy > 0 { Direction::Down } else { Direction::Up }
                        }));
                    }
                }
            }
        }

        println!("Choosing action. Goal: {:?}, Path length: {:?}", self.current_goal, self.current_path.as_ref().map(|p| p.len()));

        // 2. If we have a path, follow it
        if let Some(path) = &mut self.current_path {
            if !path.is_empty() {
                let next_pos = path.remove(0);
                let dx = next_pos.0 as i32 - player_pos.0 as i32;
                let dy = next_pos.1 as i32 - player_pos.1 as i32;

                return Ok(Action::Move(if dx > 0 { Direction::Right }
                                       else if dx < 0 { Direction::Left }
                                       else if dy > 0 { Direction::Down }
                                       else { Direction::Up }));
            } else {
                self.current_path = None; // Path is finished
            }
        }

        // 3. If we have a goal, and we are at the goal location, act on it
        if let Some(goal) = &self.current_goal {
             match goal {
                Goal::GatherResource { resource_char, .. } => {
                    if let Some(memory_tile) = &mut self.mental_map[player_pos.1 as usize][player_pos.0 as usize] {
                         if memory_tile.tile.tile_type == *resource_char {
                            self.current_goal = None; // Goal achieved
                            memory_tile.resource_richness = memory_tile.resource_richness * 0.5 + 3.0 * 0.5; // Update richness
                            return Ok(Action::Gather);
                        }
                    }
                },
                Goal::FleeFromHostile { .. } => {
                    // We are no longer near the hostile player, so we can stop fleeing
                    self.current_goal = None;
                }
            }
        }

        // 4. If we have a goal but no path, find a path
        if let Some(goal) = &self.current_goal {
            match goal {
                Goal::GatherResource { resource_char, .. } => {
                    if let Some(resource_pos) = self.find_best_resource_spot(player_pos, *resource_char) {
                        let mut grid_for_pathfinding = vec![vec![Tile::new('X'); WIDTH as usize]; HEIGHT as usize];
                        for y in 0..HEIGHT {
                            for x in 0..WIDTH {
                                if let Some(memory_tile) = &self.mental_map[y as usize][x as usize] {
                                    grid_for_pathfinding[y as usize][x as usize] = memory_tile.tile.clone();
                                }
                            }
                        }

                        if let Some(path) = pathfinding::find_path(player_pos, resource_pos, &grid_for_pathfinding) {
                            self.current_path = Some(path);
                            return self.choose_action(state, player_pos);
                        }
                    }
                },
                Goal::FleeFromHostile { .. } => { /* Already handled above */ }
            }
        }

        // 4. If no goal, set a new goal
        if self.current_goal.is_none() {
            // Find the best resource to gather based on memory
            let mut best_resource_char = 'T';
            let mut best_resource_name = "wood".to_string();
            let mut max_richness = 0.0;

            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    if let Some(memory_tile) = &self.mental_map[y as usize][x as usize] {
                        if memory_tile.resource_richness > max_richness {
                            max_richness = memory_tile.resource_richness;
                            best_resource_char = memory_tile.tile.tile_type;
                            // This is a simplification, we should have a map from char to name
                            best_resource_name = match best_resource_char {
                                'T' => "wood".to_string(),
                                'R' => "stone".to_string(),
                                'U' => "sulfur".to_string(),
                                'I' => "iron_ore".to_string(),
                                _ => "unknown".to_string(),
                            };
                        }
                    }
                }
            }

            self.current_goal = Some(Goal::GatherResource { resource_char: best_resource_char, resource_name: best_resource_name });
            // Recursively call choose_action to start planning
            return self.choose_action(state, player_pos);
        }

        // Fallback to random action if no other logic applies
        let index = rand::thread_rng().gen_range(0..self.actions.len());
        Ok(self.actions[index].clone())
    }

    pub fn record_attack_from(&mut self, attacker_id: u32) {
        println!("Player {} is now hostile!", attacker_id);
        let memory = self.player_memories.entry(attacker_id).or_insert(PlayerMemory {
            last_seen_location: None,
            relationship: RelationshipStatus::Neutral,
        });
        memory.relationship = RelationshipStatus::Hostile;
    }

    pub fn update_q_table(&mut self, state: &StateKey, action: &Action, reward: f64, next_state: &StateKey) -> Result<(), SimulationError> {
        // This function can be adapted later to work with the new goal-oriented system
        // For now, we can leave it as is, or disable it
        Ok(())
    }
}
