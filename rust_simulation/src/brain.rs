use std::collections::HashMap;
use rand::Rng;
use super::state::StateKey;
use super::errors::SimulationError;
use super::config::{WIDTH, HEIGHT};
use std::cmp::Ordering;
use super::actions::{Action, Direction};
use super::map::Tile;
use super::pathfinding;
use super::recipes::RecipeManager;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

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
    pub recipe_manager: Arc<RecipeManager>,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub epsilon: f64,
    pub goal_q_table: HashMap<String, HashMap<Goal, f64>>,
    pub mental_map: Vec<Vec<Option<MemoryTile>>>,
    pub player_memories: HashMap<u32, PlayerMemory>,
    pub current_goal: Option<Goal>,
    pub goal_stack: Vec<Goal>,
    pub current_path: Option<Vec<(u32, u32)>>,
    pub goal_commitment_ticks: u32,
}

impl Brain {
    pub fn new(actions: Vec<Action>, recipe_manager: Arc<RecipeManager>, learning_rate: f64, discount_factor: f64, epsilon: f64) -> Self {
        let goals = vec![
            Goal::GatherResource("wood".to_string()),
            Goal::GatherResource("stone".to_string()),
            Goal::CraftItem("stone_axe".to_string()),
        ];
        Brain {
            actions,
            goals,
            recipe_manager,
            learning_rate,
            discount_factor,
            epsilon,
            goal_q_table: HashMap::new(),
            mental_map: vec![vec![None; WIDTH as usize]; HEIGHT as usize],
            player_memories: HashMap::new(),
            current_goal: None,
            goal_stack: Vec::new(),
            current_path: None,
            goal_commitment_ticks: 0,
        }
    }

    pub fn choose_goal(&self, state: &HighLevelState) -> Result<Goal, SimulationError> {
        println!("Choosing a new goal...");
        let valid_goals: Vec<_> = self.goals.iter().filter(|g| self.is_goal_valid(g)).cloned().collect();
        if valid_goals.is_empty() {
            return Ok(Goal::Flee); // Fallback goal
        }

        if rand::thread_rng().r#gen::<f64>() < self.epsilon {
            let index = rand::thread_rng().gen_range(0..valid_goals.len());
            Ok(valid_goals[index].clone())
        } else {
            let state_key_str = serde_json::to_string(state)?;
            if let Some(q_values) = self.goal_q_table.get(&state_key_str) {
                q_values.iter()
                    .filter(|(g, _)| self.is_goal_valid(g))
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(Ordering::Equal))
                    .map(|(goal, _)| goal.clone())
                    .ok_or_else(|| SimulationError::Other("No best goal found".to_string()))
            } else {
                let index = rand::thread_rng().gen_range(0..valid_goals.len());
                Ok(valid_goals[index].clone())
            }
        }
    }

    pub fn update_goal_q_table(&mut self, state: &HighLevelState, goal: &Goal, reward: f64, next_state: &HighLevelState) -> Result<(), SimulationError> {
        let state_key_str = serde_json::to_string(state)?;
        let next_state_key_str = serde_json::to_string(next_state)?;
        let old_value = self.goal_q_table.get(&state_key_str).and_then(|goals| goals.get(goal)).cloned().unwrap_or(0.0);
        let next_max = self.goal_q_table.get(&next_state_key_str).map_or(0.0, |goals| goals.values().cloned().fold(f64::NEG_INFINITY, f64::max));
        let new_value = old_value + self.learning_rate * (reward + self.discount_factor * next_max - old_value);
        self.goal_q_table.entry(state_key_str).or_insert_with(HashMap::new).insert(goal.clone(), new_value);
        Ok(())
    }

    pub fn is_goal_complete(&self, player: &super::player::Player, goal: &Goal) -> bool {
        match goal {
            Goal::GatherResource(resource) => {
                if let Some(parent_goal) = self.goal_stack.last() {
                    if let Goal::CraftItem(item_name) = parent_goal {
                        let recipe = self.recipe_manager.get_required_resources(item_name, 1);
                        if let Some(&required_amount) = recipe.get(resource) {
                            return player.get_total_quantity(resource) >= required_amount;
                        }
                    }
                }
                player.get_total_quantity(resource) > 10 // Default
            },
            Goal::CraftItem(item) => player.get_total_quantity(item) > 0,
            _ => false,
        }
    }

    pub fn tick(&mut self, player: &super::player::Player, high_level_state: &HighLevelState, current_episode: u32) -> Result<Action, SimulationError> {
        if let Some(action) = self.handle_threats(player) {
            return Ok(action);
        }

        if self.goal_commitment_ticks > 0 {
            self.goal_commitment_ticks -= 1;
        }

        if let Some(goal) = &self.current_goal {
            if self.is_goal_complete(player, goal) || !self.is_goal_valid(goal) {
                self.current_goal = None;
                self.current_path = None;
                self.goal_commitment_ticks = 0;
            }
        }

        if self.current_goal.is_none() && self.goal_commitment_ticks == 0 {
            self.current_goal = Some(self.choose_goal(high_level_state)?);
            self.goal_commitment_ticks = 10; // Commit to the new goal for 10 ticks
        }

        self.choose_action_for_goal(player, current_episode)
    }

    fn is_goal_valid(&self, goal: &Goal) -> bool {
        match goal {
            Goal::GatherResource(resource_name) => {
                let resource_char = self.resource_name_to_char(resource_name);
                self.mental_map.iter().any(|row| row.iter().any(|tile| tile.as_ref().map_or(false, |t| t.tile.tile_type == resource_char)))
            },
            _ => true,
        }
    }

    fn handle_threats(&mut self, player: &super::player::Player) -> Option<Action> {
        for memory in self.player_memories.values() {
            if memory.relationship == RelationshipStatus::Hostile {
                if let Some(loc) = memory.last_seen_location {
                    let dist = ((player.x as f64 - loc.0 as f64).powi(2) + (player.y as f64 - loc.1 as f64).powi(2)).sqrt();
                    if dist < 5.0 {
                        self.current_goal = Some(Goal::Flee);
                        self.current_path = None;
                        let dx = player.x as i32 - loc.0 as i32;
                        let dy = player.y as i32 - loc.1 as i32;
                        return Some(Action::Move(if dx.abs() > dy.abs() {
                            if dx > 0 { Direction::Right } else { Direction::Left }
                        } else {
                            if dy > 0 { Direction::Down } else { Direction::Up }
                        }));
                    }
                }
            }
        }
        None
    }

    fn choose_action_for_goal(&mut self, player: &super::player::Player, current_episode: u32) -> Result<Action, SimulationError> {
        if let Some(path) = &mut self.current_path {
            if !path.is_empty() {
                let next_pos = path.remove(0);
                let dx = next_pos.0 as i32 - player.x as i32;
                let dy = next_pos.1 as i32 - player.y as i32;
                return Ok(Action::Move(if dx > 0 { Direction::Right } else if dx < 0 { Direction::Left } else if dy > 0 { Direction::Down } else { Direction::Up }));
            } else {
                self.current_path = None;
            }
        }

        if let Some(goal) = self.current_goal.clone() {
            match goal {
                Goal::GatherResource(resource_name) => return self.execute_gather_goal(player, &resource_name, current_episode),
                Goal::CraftItem(item_name) => return self.execute_craft_item_goal(player, &item_name, current_episode),
                _ => {}
            }
        }

        Ok(self.actions[0].clone())
    }

    fn execute_gather_goal(&mut self, player: &super::player::Player, resource_name: &str, current_episode: u32) -> Result<Action, SimulationError> {
        let resource_char = self.resource_name_to_char(resource_name);
        let player_pos = (player.x, player.y);

        if let Some(memory_tile) = &mut self.mental_map[player_pos.1 as usize][player_pos.0 as usize] {
            if memory_tile.tile.tile_type == resource_char {
                memory_tile.resource_richness = memory_tile.resource_richness * 0.5 + 3.0 * 0.5;
                return Ok(Action::Gather);
            }
        }

        if let Some(resource_pos) = self.find_best_resource_spot(player_pos, resource_char, current_episode) {
            if let Some(path) = pathfinding::find_path(player_pos, resource_pos, &self.mental_map) {
                self.current_path = Some(path);
                return self.choose_action_for_goal(player, current_episode);
            }
        }

        // If we reach here, it means we couldn't find a path or the resource doesn't exist.
        // Clear the goal to avoid getting stuck.
        self.current_goal = None;
        Ok(self.actions[0].clone()) // Fallback action
    }

    fn execute_craft_item_goal(&mut self, player: &super::player::Player, item_name: &str, current_episode: u32) -> Result<Action, SimulationError> {
        let recipe = self.recipe_manager.get_required_resources(item_name, 1);
        let mut missing_resource = None;

        for (resource, &required_amount) in &recipe {
            if player.get_total_quantity(resource) < required_amount {
                missing_resource = Some(resource.clone());
                break;
            }
        }

        if let Some(resource) = missing_resource {
            // We are missing a resource, so we need to gather it.
            // Push the current CraftItem goal onto the stack.
            if let Some(craft_goal) = self.current_goal.clone() {
                self.goal_stack.push(craft_goal);
            }
            // Set the new goal to gather the missing resource.
            self.current_goal = Some(Goal::GatherResource(resource));
            return self.choose_action_for_goal(player, current_episode);
        } else {
            // We have all the resources, so we can craft the item.
            self.current_goal = self.goal_stack.pop(); // Go back to the parent goal
            return Ok(Action::Craft(item_name.to_string()));
        }
    }

    fn resource_name_to_char(&self, resource_name: &str) -> char {
        match resource_name {
            "wood" => 'T',
            "stone" => 'R',
            _ => 'X',
        }
    }

    fn find_best_resource_spot(&self, player_pos: (u32, u32), resource_char: char, current_episode: u32) -> Option<(u32, u32)> {
        let mut best_score = f64::MIN;
        let mut best_pos = None;

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if let Some(memory_tile) = &self.mental_map[y as usize][x as usize] {
                    if memory_tile.tile.tile_type == resource_char {
                        let dist = ((player_pos.0 as f64 - x as f64).powi(2) + (player_pos.1 as f64 - y as f64).powi(2)).sqrt();
                        let time_since_seen = (current_episode - memory_tile.last_seen_episode) as f64;
                        // Score now also considers how recently the resource was seen.
                        let score = memory_tile.resource_richness as f64 / ((dist + 1.0) * (time_since_seen + 1.0));
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
        let memory = self.player_memories.entry(attacker_id).or_insert(PlayerMemory {
            last_seen_location: None,
            relationship: RelationshipStatus::Neutral,
        });
        memory.relationship = RelationshipStatus::Hostile;
    }

}
