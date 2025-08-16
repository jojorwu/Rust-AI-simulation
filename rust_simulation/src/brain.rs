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
}

pub struct Brain {
    pub actions: Vec<Action>,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub epsilon: f64,
    pub q_table: HashMap<String, HashMap<Action, f64>>,
    pub mental_map: Vec<Vec<Option<Tile>>>,
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
            current_goal: None,
            current_path: None,
        }
    }

    fn find_closest_resource(&self, player_pos: (u32, u32), resource_char: char) -> Option<(u32, u32)> {
        let mut closest_dist = f64::MAX;
        let mut closest_pos = None;

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if let Some(tile) = &self.mental_map[y as usize][x as usize] {
                    if tile.tile_type == resource_char {
                        let dist = ((player_pos.0 as f64 - x as f64).powi(2) + (player_pos.1 as f64 - y as f64).powi(2)).sqrt();
                        if dist < closest_dist {
                            closest_dist = dist;
                            closest_pos = Some((x, y));
                        }
                    }
                }
            }
        }
        closest_pos
    }

    pub fn choose_action(&mut self, state: &StateKey, player_pos: (u32, u32)) -> Result<Action, SimulationError> {
        println!("Choosing action. Goal: {:?}, Path length: {:?}", self.current_goal, self.current_path.as_ref().map(|p| p.len()));
        // 1. If we have a path, follow it
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

        // 2. If we have a goal, and we are at the goal location, act on it
        if let Some(goal) = &self.current_goal {
             match goal {
                Goal::GatherResource { resource_char, .. } => {
                    // Check if player is at a tile with the resource
                    if let Some(tile) = &self.mental_map[player_pos.1 as usize][player_pos.0 as usize] {
                         if tile.tile_type == *resource_char {
                            self.current_goal = None; // Goal achieved
                            return Ok(Action::Gather);
                        }
                    }
                }
            }
        }

        // 3. If we have a goal but no path, find a path
        if let Some(goal) = &self.current_goal {
            match goal {
                Goal::GatherResource { resource_char, .. } => {
                    if let Some(resource_pos) = self.find_closest_resource(player_pos, *resource_char) {
                        // Create a temporary grid for pathfinding from the mental map
                        let mut grid_for_pathfinding = vec![vec![Tile::new(' '); WIDTH as usize]; HEIGHT as usize];
                        for y in 0..HEIGHT {
                            for x in 0..WIDTH {
                                grid_for_pathfinding[y as usize][x as usize] = self.mental_map[y as usize][x as usize].clone().unwrap_or(Tile::new('X'));
                            }
                        }

                        if let Some(path) = pathfinding::find_path(player_pos, resource_pos, &grid_for_pathfinding) {
                            self.current_path = Some(path);
                            // Recursively call choose_action to immediately start moving
                            return self.choose_action(state, player_pos);
                        }
                    }
                }
            }
        }

        // 4. If no goal, set a new goal
        if self.current_goal.is_none() {
            // Simple logic: just gather wood for now
            self.current_goal = Some(Goal::GatherResource { resource_char: 'T', resource_name: "wood".to_string() });
            // Recursively call choose_action to start planning
            return self.choose_action(state, player_pos);
        }

        // Fallback to random action if no other logic applies
        let index = rand::thread_rng().gen_range(0..self.actions.len());
        Ok(self.actions[index].clone())
    }

    pub fn update_q_table(&mut self, state: &StateKey, action: &Action, reward: f64, next_state: &StateKey) -> Result<(), SimulationError> {
        // This function can be adapted later to work with the new goal-oriented system
        // For now, we can leave it as is, or disable it
        Ok(())
    }
}
