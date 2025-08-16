use std::collections::HashMap;
use rand::Rng;
use super::state::StateKey;
use super::errors::SimulationError;
use super::config::{WIDTH, HEIGHT};
use super::actions::{Action, Direction};
use super::map::Tile;
use super::pathfinding;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Goal {
    GatherResource(String),
    CraftItem(String),
    AttackPlayer(u32),
    Flee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighLevelState {
    pub has_wood: bool,
    pub has_stone: bool,
    pub has_iron_ore: bool,
    pub has_stone_axe: bool,
    pub num_hostile_players: u32,
    pub health_level: u32,
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
    pub goals: Vec<Goal>,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub epsilon: f64,
    pub q_table: HashMap<String, HashMap<Action, f64>>,
    pub goal_q_table: HashMap<String, HashMap<Goal, f64>>,
    pub mental_map: Vec<Vec<Option<MemoryTile>>>,
    pub player_memories: HashMap<u32, PlayerMemory>,
    pub current_goal: Option<Goal>,
    pub current_path: Option<Vec<(u32, u32)>>,
}

impl Brain {
    pub fn new(actions: Vec<Action>, learning_rate: f64, discount_factor: f64, epsilon: f64) -> Self {
        let goals = vec![
            Goal::GatherResource("wood".to_string()),
            Goal::GatherResource("stone".to_string()),
            Goal::CraftItem("stone_axe".to_string()),
        ];
        Brain {
            actions,
            goals,
            learning_rate,
            discount_factor,
            epsilon,
            q_table: HashMap::new(),
            goal_q_table: HashMap::new(),
            mental_map: vec![vec![None; WIDTH as usize]; HEIGHT as usize],
            player_memories: HashMap::new(),
            current_goal: None,
            current_path: None,
        }
    }

    pub fn choose_goal(&self, state: &HighLevelState) -> Result<Goal, SimulationError> {
        println!("Choosing a new goal...");
        if rand::thread_rng().r#gen::<f64>() < self.epsilon {
            // Explore
            let index = rand::thread_rng().gen_range(0..self.goals.len());
            Ok(self.goals[index].clone())
        } else {
            // Exploit
            let state_key_str = serde_json::to_string(state)?;
            if let Some(q_values) = self.goal_q_table.get(&state_key_str) {
                let best_goal = q_values
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(goal, _)| goal.clone())
                    .unwrap_or_else(|| self.goals[0].clone());
                Ok(best_goal)
            } else {
                // If state is unknown, choose randomly
                let index = rand::thread_rng().gen_range(0..self.goals.len());
                Ok(self.goals[index].clone())
            }
        }
    }

    pub fn update_goal_q_table(&mut self, state: &HighLevelState, goal: &Goal, reward: f64, next_state: &HighLevelState) -> Result<(), SimulationError> {
        let state_key_str = serde_json::to_string(state)?;
        let next_state_key_str = serde_json::to_string(next_state)?;

        let old_value = self.goal_q_table
            .get(&state_key_str)
            .and_then(|goals| goals.get(goal))
            .cloned()
            .unwrap_or(0.0);

        let next_max = self.goal_q_table
            .get(&next_state_key_str)
            .map_or(0.0, |goals| {
                goals.values().cloned().fold(f64::NEG_INFINITY, f64::max)
            });

        let new_value = old_value + self.learning_rate * (reward + self.discount_factor * next_max - old_value);

        self.goal_q_table
            .entry(state_key_str)
            .or_insert_with(HashMap::new)
            .insert(goal.clone(), new_value);

        Ok(())
    }

    pub fn is_goal_complete(&self, player: &super::player::Player, goal: &Goal) -> bool {
        match goal {
            Goal::GatherResource(resource) => player.get_total_quantity(resource) > 10,
            Goal::CraftItem(item) => player.get_total_quantity(item) > 0,
            _ => false, // Flee and Attack goals are handled differently
        }
    }

    pub fn tick(&mut self, player: &super::player::Player, high_level_state: &HighLevelState) -> Result<Action, SimulationError> {
        // 1. Check if current goal is complete
        if let Some(goal) = &self.current_goal {
            if self.is_goal_complete(player, goal) {
                // For now, let's just clear the goal. We'll add reward logic later.
                self.current_goal = None;
                self.current_path = None;
            }
        }

        // 2. If no goal, choose a new one
        if self.current_goal.is_none() {
            let new_goal = self.choose_goal(high_level_state)?;
            self.current_goal = Some(new_goal);
        }

        // 3. Choose a low-level action to work towards the goal
        self.choose_action_for_goal(player)
    }

    fn choose_action_for_goal(&mut self, player: &super::player::Player) -> Result<Action, SimulationError> {
        let player_pos = (player.x, player.y);

        // Fleeing is a high-priority, reactive action
        for (_id, memory) in &self.player_memories {
            if memory.relationship == RelationshipStatus::Hostile {
                if let Some(loc) = memory.last_seen_location {
                    let dist = ((player_pos.0 as f64 - loc.0 as f64).powi(2) + (player_pos.1 as f64 - loc.1 as f64).powi(2)).sqrt();
                    if dist < 5.0 {
                        let dx = player_pos.0 as i32 - loc.0 as i32;
                        let dy = player_pos.1 as i32 - loc.1 as i32;
                        return Ok(Action::Move(if dx.abs() > dy.abs() {
                            if dx > 0 { Direction::Right } else { Direction::Left }
                        } else {
                            if dy > 0 { Direction::Down } else { Direction::Up }
                        }));
                    }
                }
            }
        }

        // Follow path if one exists
        if let Some(path) = &mut self.current_path {
            if !path.is_empty() {
                let next_pos = path.remove(0);
                let dx = next_pos.0 as i32 - player_pos.0 as i32;
                let dy = next_pos.1 as i32 - player_pos.1 as i32;
                return Ok(Action::Move(if dx > 0 { Direction::Right } else if dx < 0 { Direction::Left } else if dy > 0 { Direction::Down } else { Direction::Up }));
            } else {
                self.current_path = None;
            }
        }

        // If at destination or no path, take action based on goal
        if let Some(goal) = &self.current_goal {
            match goal {
                Goal::GatherResource(resource_name) => {
                    let resource_char = match resource_name.as_str() {
                        "wood" => 'T',
                        "stone" => 'R',
                        _ => 'X',
                    };
                    if let Some(memory_tile) = &mut self.mental_map[player_pos.1 as usize][player_pos.0 as usize] {
                        if memory_tile.tile.tile_type == resource_char {
                            memory_tile.resource_richness = memory_tile.resource_richness * 0.5 + 3.0 * 0.5;
                            return Ok(Action::Gather);
                        }
                    }
                    // If not at resource, find path
                    if let Some(resource_pos) = self.find_best_resource_spot(player_pos, resource_char) {
                        let mut grid = vec![vec![Tile::new('X'); WIDTH as usize]; HEIGHT as usize];
                        for y in 0..HEIGHT{
                            for x in 0..WIDTH{
                                if let Some(mem_tile) = &self.mental_map[y as usize][x as usize]{
                                    grid[y as usize][x as usize] = mem_tile.tile.clone();
                                }
                            }
                        }
                        if let Some(path) = pathfinding::find_path(player_pos, resource_pos, &grid) {
                            self.current_path = Some(path);
                            return self.choose_action_for_goal(player);
                        }
                    }
                },
                Goal::CraftItem(item_name) => {
                    // For now, just try to craft it. In the future, could check for resources first.
                    return Ok(Action::Craft(item_name.clone()));
                },
                _ => {} // Other goals
            }
        }

        // Fallback
        Ok(self.actions[0].clone())
    }

    fn find_best_resource_spot(&self, player_pos: (u32, u32), resource_char: char) -> Option<(u32, u32)> {
        let mut best_score = f64::MIN;
        let mut best_pos = None;

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if let Some(memory_tile) = &self.mental_map[y as usize][x as usize] {
                    if memory_tile.tile.tile_type == resource_char {
                        let dist = ((player_pos.0 as f64 - x as f64).powi(2) + (player_pos.1 as f64 - y as f64).powi(2)).sqrt();
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

    pub fn record_attack_from(&mut self, attacker_id: u32) {
        println!("Player {} is now hostile!", attacker_id);
        let memory = self.player_memories.entry(attacker_id).or_insert(PlayerMemory {
            last_seen_location: None,
            relationship: RelationshipStatus::Neutral,
        });
        memory.relationship = RelationshipStatus::Hostile;
    }

    pub fn update_q_table(&mut self, state: &StateKey, action: &Action, reward: f64, next_state: &StateKey) -> Result<(), SimulationError> {
        Ok(())
    }
}
