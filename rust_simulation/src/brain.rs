use std::collections::HashMap;
use rand::Rng;
use super::errors::SimulationError;
use super::config::{WIDTH, HEIGHT};
use std::cmp::Ordering;
use super::actions::{Action};
use super::map::Tile;
use super::pathfinding;
use crate::components::{WantsToGather, WantsToCraft, WantsToBuild};
use super::recipes::RecipeManager;
use super::ecs::{World, Entity};
use super::components::Position;
use super::player::Player;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Goal {
    GatherResource(String),
    CraftItem(String),
    Build(String),
    Attack(u32),
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
    pub _last_seen_episode: u32,
    pub _resource_richness: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipStatus {
    _Neutral,
    Hostile,
}

#[derive(Debug, Clone)]
pub struct PlayerMemory {
    pub _last_seen_location: Option<(u32, u32)>,
    pub relationship: RelationshipStatus,
}

pub struct Brain {
    pub _actions: Vec<Action>,
    pub goals: Vec<Goal>,
    pub recipe_manager: Arc<RecipeManager>,
    pub _learning_rate: f64,
    pub _discount_factor: f64,
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
    pub fn new(actions: Vec<Action>, recipe_manager: Arc<RecipeManager>, _learning_rate: f64, _discount_factor: f64, epsilon: f64) -> Self {
        let goals = vec![
            Goal::GatherResource("wood".to_string()),
            Goal::GatherResource("stone".to_string()),
            Goal::CraftItem("stone_axe".to_string()),
            Goal::Build("foundation".to_string()),
            Goal::Attack(1), // Attack player 1
        ];
        Brain {
            _actions: actions,
            goals,
            recipe_manager,
            _learning_rate,
            _discount_factor,
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


    pub fn is_goal_complete(&self, world: &World, entity: Entity, goal: &Goal) -> bool {
        let player = world.get_component::<Player>(entity).unwrap();
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

    pub fn tick(&mut self, world: &mut World, entity: Entity, high_level_state: &HighLevelState, current_episode: u32) -> Result<(), SimulationError> {
        // if let Some(_action) = self.handle_threats(world, entity) {
        //     // return Ok(action);
        // }

        if self.goal_commitment_ticks > 0 {
            self.goal_commitment_ticks -= 1;
        }

        if let Some(goal) = &self.current_goal {
            if self.is_goal_complete(world, entity, goal) || !self.is_goal_valid(goal) {
                self.current_goal = None;
                self.current_path = None;
                self.goal_commitment_ticks = 0;
            }
        }

        if self.current_goal.is_none() && self.goal_commitment_ticks == 0 {
            self.current_goal = Some(self.choose_goal(high_level_state)?);
            self.goal_commitment_ticks = 10; // Commit to the new goal for 10 ticks
        }

        self.choose_action_for_goal(world, entity, current_episode)
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


    fn choose_action_for_goal(&mut self, world: &mut World, entity: Entity, current_episode: u32) -> Result<(), SimulationError> {
        if let Some(path) = &mut self.current_path {
            if !path.is_empty() {
                // let next_pos = path.remove(0);
                // let _dx = next_pos.0 as i32 - player_pos.x as i32;
                // let _dy = next_pos.1 as i32 - player_pos.y as i32;
                // return Ok(Action::Move(if dx > 0 { Direction::Right } else if dx < 0 { Direction::Left } else if dy > 0 { Direction::Down } else { Direction::Up }));
            } else {
                self.current_path = None;
            }
        }

        if let Some(goal) = self.current_goal.clone() {
            match goal {
                Goal::GatherResource(resource_name) => self.execute_gather_goal(world, entity, &resource_name, current_episode)?,
                Goal::CraftItem(item_name) => self.execute_craft_item_goal(world, entity, &item_name, current_episode)?,
                Goal::Build(structure_name) => self.execute_build_goal(world, entity, &structure_name, current_episode)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn execute_gather_goal(&mut self, world: &mut World, entity: Entity, resource_name: &str, current_episode: u32) -> Result<(), SimulationError> {
        let resource_char = self.resource_name_to_char(resource_name);
        let player_pos = *world.get_component::<Position>(entity).unwrap();

        // Find the target resource
        let mut target_entity = None;
        for e in 0..world.entities.len() {
            if let Some(resource) = world.get_component::<super::components::Resource>(e) {
                if resource.resource_type == resource_char {
                    target_entity = Some(e);
                    break;
                }
            }
        }

        if let Some(target) = target_entity {
            let target_pos = world.get_component::<Position>(target).unwrap();
            let dx = (player_pos.x as i32 - target_pos.x as i32).abs();
            let dy = (player_pos.y as i32 - target_pos.y as i32).abs();

            if dx <= 1 && dy <= 1 {
                world.add_component(entity, WantsToGather { target });
            } else {
                if let Some(path) = pathfinding::find_path((player_pos.x, player_pos.y), (target_pos.x, target_pos.y), &self.mental_map) {
                    self.current_path = Some(path);
                    return self.choose_action_for_goal(world, entity, current_episode);
                }
            }
        } else {
            // If we reach here, it means we couldn't find a path or the resource doesn't exist.
            // Clear the goal to avoid getting stuck.
            self.current_goal = None;
        }

        Ok(())
    }

    fn execute_craft_item_goal(&mut self, world: &mut World, entity: Entity, item_name: &str, current_episode: u32) -> Result<(), SimulationError> {
        let recipe = self.recipe_manager.get_required_resources(item_name, 1);
        let mut missing_resource = None;
        let player = world.get_component::<Player>(entity).unwrap();

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
            return self.choose_action_for_goal(world, entity, current_episode);
        } else {
            // We have all the resources, so we can craft the item.
            self.current_goal = self.goal_stack.pop(); // Go back to the parent goal
            world.add_component(entity, WantsToCraft { item_name: item_name.to_string() });
            Ok(())
        }
    }

    fn execute_build_goal(&mut self, world: &mut World, entity: Entity, structure_name: &str, _current_episode: u32) -> Result<(), SimulationError> {
        world.add_component(entity, WantsToBuild { structure_name: structure_name.to_string() });
        Ok(())
    }

    fn resource_name_to_char(&self, resource_name: &str) -> char {
        match resource_name {
            "wood" => 'T',
            "stone" => 'R',
            _ => 'X',
        }
    }



}
