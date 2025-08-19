use rand::Rng;
use super::errors::SimulationError;
use super::config::{WIDTH, HEIGHT};
use std::cmp::Ordering;
use super::map::Tile;
use super::pathfinding;
use crate::components::{Position, WantsToGather, WantsToCraft, WantsToBuild, Resource, Inventory};
use std::collections::{HashMap, HashSet};
use super::recipes::RecipeManager;
use super::ecs::{World, Entity};
use super::player::Player;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::fs;
use std::env;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Goal {
    GatherResource(String),
    CraftItem(String),
    Build(String),
    Attack(u32),
    Flee,
    Explore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighLevelState {
    pub has_wood: bool,
    pub has_stone: bool,
    pub has_iron_ore: bool,
    pub has_stone_axe: bool,
    pub num_hostile_players: u32,
    pub health_level: u32,
    pub is_night: bool,
}

#[derive(Debug, Clone)]
pub struct MemoryTile {
    pub tile: Tile,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipStatus {
    Hostile,
}

#[derive(Debug, Clone)]
pub struct PlayerMemory {
    pub relationship: RelationshipStatus,
}

/// The Brain struct represents the AI for an agent in the simulation.
/// It uses a goal-oriented architecture to make decisions.
pub struct Brain {
    pub goals: Vec<Goal>,
    pub recipe_manager: Arc<RecipeManager>,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub epsilon: f64,
    pub goal_q_table: HashMap<String, HashMap<Goal, f64>>,
    pub mental_map: Vec<Vec<Option<MemoryTile>>>,
    pub known_resources: HashMap<String, HashSet<Position>>,
    pub player_memories: HashMap<u32, PlayerMemory>,
    pub current_goal: Option<Goal>,
    pub goal_stack: Vec<Goal>,
    pub current_path: Option<Vec<(u32, u32)>>,
    pub goal_commitment_ticks: u32,
    pub prev_state: Option<HighLevelState>,
    pub prev_goal: Option<Goal>,
}

impl Brain {
    pub fn new(recipe_manager: Arc<RecipeManager>, learning_rate: f64, discount_factor: f64, epsilon: f64) -> Self {
        let goals = vec![
            Goal::GatherResource("wood".to_string()),
            Goal::GatherResource("stone".to_string()),
            Goal::CraftItem("stone_axe".to_string()),
            Goal::Build("foundation".to_string()),
        ];

        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let q_table_path = std::path::Path::new(manifest_dir).join("../q_table.json");
        let goal_q_table = if let Ok(file) = fs::read_to_string(q_table_path) {
            serde_json::from_str(&file).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Brain {
            goals,
            recipe_manager,
            learning_rate,
            discount_factor,
            epsilon,
            goal_q_table,
            mental_map: vec![vec![None; WIDTH as usize]; HEIGHT as usize],
            known_resources: HashMap::new(),
            player_memories: HashMap::new(),
            current_goal: None,
            goal_stack: Vec::new(),
            current_path: None,
            goal_commitment_ticks: 0,
            prev_state: None,
            prev_goal: None,
        }
    }

    pub fn save_q_table(&self) -> Result<(), SimulationError> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let q_table_path = std::path::Path::new(manifest_dir).join("../q_table.json");
        let json = serde_json::to_string_pretty(&self.goal_q_table)?;
        fs::write(q_table_path, json)?;
        Ok(())
    }

    pub fn choose_goal(&self, world: &World, state: &HighLevelState) -> Result<Goal, SimulationError> {
        let valid_goals: Vec<_> = self.goals.iter().filter(|g| self.is_goal_valid(world, g)).cloned().collect();
        if valid_goals.is_empty() {
            return Ok(Goal::Flee); // Fallback goal
        }

        let choose_random_goal = || {
            let index = rand::thread_rng().gen_range(0..valid_goals.len());
            Ok(valid_goals[index].clone())
        };

        if rand::thread_rng().r#gen::<f64>() < self.epsilon {
            return choose_random_goal();
        }

        let state_key_str = serde_json::to_string(state)?;
        if let Some(q_values) = self.goal_q_table.get(&state_key_str) {
            let mut modified_q_values = q_values.clone();
            if state.is_night {
                for (goal, q_value) in modified_q_values.iter_mut() {
                    if let Goal::Build(_) = goal {
                        *q_value += 10.0; // Add a large bonus for building at night
                    }
                }
            }

            modified_q_values.iter()
                .filter(|(g, _)| self.is_goal_valid(world, g))
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(Ordering::Equal))
                .map(|(goal, _)| goal.clone())
                .map(Ok) // wrap in Result
                .unwrap_or_else(choose_random_goal) // if max_by is None, choose random
        } else {
            choose_random_goal()
        }
    }

    pub fn is_goal_complete(&self, world: &World, entity: Entity, goal: &Goal) -> bool {
        if let Some(inventory) = world.get_component::<Inventory>(entity) {
            match goal {
                Goal::GatherResource(resource) => {
                    if let Some(parent_goal) = self.goal_stack.last() {
                        if let Goal::CraftItem(item_name) = parent_goal {
                            let recipe = self.recipe_manager.get_required_resources(item_name, 1);
                            if let Some(&required_amount) = recipe.get(resource) {
                                return inventory.get_quantity(resource) >= required_amount;
                            }
                        }
                    }
                    inventory.get_quantity(resource) > 10 // Default
                },
                Goal::CraftItem(item) => inventory.has_item(item, 1),
                Goal::Explore => {
                    // Considered complete if we have arrived at our random destination.
                    // The planner will then try the next step (gathering), which might trigger another explore goal
                    // if the resource wasn't found on the way.
                    return self.current_path.as_ref().map_or(true, |p| p.is_empty());
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn plan_goal(&self, world: &World, entity: Entity, goal: &Goal) -> Result<Vec<Goal>, SimulationError> {
        let mut plan = Vec::new();
        match goal {
            Goal::CraftItem(item_name) => {
                let required = self.recipe_manager.get_required_resources(item_name, 1);
                let inventory = world.get_component::<Inventory>(entity);

                for (resource, &required_amount) in &required {
                    let has_enough = inventory.map_or(false, |inv| inv.get_quantity(resource) >= required_amount);
                    if !has_enough {
                        if !self.known_resources.contains_key(resource) {
                            plan.push(Goal::Explore);
                        }
                        plan.push(Goal::GatherResource(resource.clone()));
                    }
                }
                plan.push(goal.clone());
            }
            Goal::Build(structure_name) => {
                let required = self.recipe_manager.get_required_resources(structure_name, 1);
                let inventory = world.get_component::<Inventory>(entity);

                for (resource, &required_amount) in &required {
                     let has_enough = inventory.map_or(false, |inv| inv.get_quantity(resource) >= required_amount);
                    if !has_enough {
                        if !self.known_resources.contains_key(resource) {
                            plan.push(Goal::Explore);
                        }
                        plan.push(Goal::GatherResource(resource.clone()));
                    }
                }
                plan.push(goal.clone());
            }
            _ => {
                plan.push(goal.clone());
            }
        }
        Ok(plan)
    }

    pub fn tick(&mut self, world: &mut World, spatial_map: &HashMap<(u32, u32), Vec<Entity>>, entity: Entity, high_level_state: &HighLevelState, visible_tiles: &Vec<(Position, Tile)>) -> Result<(), SimulationError> {
        // Update Q-table based on the outcome of the previous action
        if let (Some(prev_state), Some(prev_goal)) = (self.prev_state.clone(), self.prev_goal.clone()) {
            let reward = if self.is_goal_complete(world, entity, &prev_goal) {
                10.0
            } else {
                -0.1
            };
            self.update_q_table(&prev_state, &prev_goal, reward, high_level_state)?;
        }

        self.update_mental_map(visible_tiles, world, spatial_map);
        self.handle_opportunities(world, entity, spatial_map, visible_tiles);
        self.update_current_goal(world, entity, high_level_state)?;
        self.choose_action_for_goal(world, spatial_map, entity, 0)?; // episode is not used anymore

        // Store the current state and goal for the next tick's Q-table update
        self.prev_state = Some(high_level_state.clone());
        self.prev_goal = self.current_goal.clone();

        Ok(())
    }

    fn handle_opportunities(&mut self, world: &World, entity: Entity, spatial_map: &HashMap<(u32, u32), Vec<Entity>>, visible_tiles: &Vec<(Position, Tile)>) {
        if self.goal_commitment_ticks >= 5 { return; } // Too committed to be distracted

        let valuable_resources = ["iron_ore"]; // Hardcoded for now

        for (pos, _tile) in visible_tiles {
            if let Some(entities_at_pos) = spatial_map.get(&(pos.x, pos.y)) {
                for &entity_id in entities_at_pos {
                    if let Some(resource) = world.get_component::<Resource>(entity_id) {
                        if valuable_resources.contains(&resource.name.as_str()) {
                            // Found a valuable resource!
                            // Is it a new opportunity? Check inventory.
                            let has_it_already = world.get_component::<Inventory>(entity)
                                .map_or(false, |inv| inv.get_quantity(&resource.name) > 0);

                            if !has_it_already {
                                // This is a good opportunity. Interrupt current plan.
                                self.goal_stack.clear();
                                self.current_path = None;
                                self.current_goal = Some(Goal::GatherResource(resource.name.clone()));
                                self.goal_commitment_ticks = 10; // Commit to this new goal
                                return; // Only take one opportunity per tick
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_mental_map(&mut self, visible_tiles: &Vec<(Position, Tile)>, world: &World, spatial_map: &HashMap<(u32, u32), Vec<Entity>>) {
        for (pos, tile) in visible_tiles {
            self.mental_map[pos.y as usize][pos.x as usize] = Some(MemoryTile {
                tile: tile.clone(),
            });

            // Optimization: Update known resource locations
            if let Some(entities_at_pos) = spatial_map.get(&(pos.x, pos.y)) {
                for &entity_id in entities_at_pos {
                    if let Some(resource) = world.get_component::<Resource>(entity_id) {
                        self.known_resources
                            .entry(resource.name.clone())
                            .or_default()
                            .insert(*pos);
                    }
                }
            }
        }
    }

    fn update_current_goal(&mut self, world: &World, entity: Entity, high_level_state: &HighLevelState) -> Result<(), SimulationError> {
        if self.handle_threats(world, entity) {
            self.goal_commitment_ticks = 5; // Commit to the threat response
            return Ok(());
        }

        if self.goal_commitment_ticks > 0 {
            self.goal_commitment_ticks -= 1;
        }

        if let Some(goal) = &self.current_goal {
            if self.is_goal_complete(world, entity, goal) {
                self.current_path = None; // Reset path for next step
                self.current_goal = self.goal_stack.pop();
            } else if !self.is_goal_valid(world, goal) {
                // Current step is invalid, so the whole plan is invalid. Replanning needed.
                self.current_goal = None;
                self.goal_stack.clear();
                self.current_path = None;
                self.goal_commitment_ticks = 0; // Force replan
            }
        }

        if self.current_goal.is_none() && self.goal_commitment_ticks == 0 {
            let new_high_level_goal = self.choose_goal(world, high_level_state)?;

            let mut plan = self.plan_goal(world, entity, &new_high_level_goal)?;
            plan.reverse();
            self.goal_stack = plan;

            self.current_goal = self.goal_stack.pop();

            if self.current_goal.is_some() {
                self.goal_commitment_ticks = 10; // Commit to the new plan
            }
        }
        Ok(())
    }

    fn is_goal_valid(&self, _world: &World, goal: &Goal) -> bool {
        match goal {
            Goal::GatherResource(resource_name) => {
                // Optimization: Check the known_resources map instead of scanning the world.
                self.known_resources.get(resource_name).map_or(false, |positions| !positions.is_empty())
            },
            _ => true,
        }
    }

    fn choose_action_for_goal(&mut self, world: &mut World, spatial_map: &HashMap<(u32, u32), Vec<Entity>>, entity: Entity, current_episode: u32) -> Result<(), SimulationError> {
        if self.follow_path(world, entity) {
            return Ok(()); // We have a move action, so we are done for this tick.
        }

        if let Some(goal) = self.current_goal.clone() {
            match goal {
                Goal::GatherResource(resource_name) => self.execute_gather_goal(world, spatial_map, entity, &resource_name, current_episode)?,
                Goal::CraftItem(item_name) => self.execute_craft_item_goal(world, entity, &item_name, current_episode)?,
                Goal::Build(structure_name) => self.execute_build_goal(world, entity, &structure_name, current_episode)?,
                Goal::Attack(target_id) => self.execute_attack_goal(world, entity, target_id, current_episode)?,
                Goal::Flee => self.execute_flee_goal(world, entity, current_episode)?,
                Goal::Explore => self.execute_explore_goal(world, entity, current_episode)?,
            }
        }

        Ok(())
    }

    fn follow_path(&mut self, world: &mut World, entity: Entity) -> bool {
        if let Some(path) = &mut self.current_path {
            if !path.is_empty() {
                if let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) {
                    let next_pos = path.remove(0);
                    let dx = next_pos.0 as i32 - player_pos.x as i32;
                    let dy = next_pos.1 as i32 - player_pos.y as i32;
                    world.add_component(entity, crate::components::Velocity { dx, dy });
                    return true; // Moved along path
                }
            } else {
                self.current_path = None;
            }
        }
        false // Did not move along path
    }

    pub fn update_q_table(&mut self, prev_state: &HighLevelState, goal: &Goal, reward: f64, new_state: &HighLevelState) -> Result<(), SimulationError> {
        let prev_state_key = serde_json::to_string(prev_state)?;
        let new_state_key = serde_json::to_string(new_state)?;

        let old_q_value = self.goal_q_table
            .get(&prev_state_key)
            .and_then(|q_values| q_values.get(goal))
            .cloned()
            .unwrap_or(0.0);

        let max_future_q = self.goal_q_table
            .get(&new_state_key)
            .map(|q_values| {
                q_values.values().cloned().max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap_or(0.0)
            })
            .unwrap_or(0.0);

        let new_q_value = old_q_value + self.learning_rate * (reward + self.discount_factor * max_future_q - old_q_value);

        self.goal_q_table
            .entry(prev_state_key)
            .or_default()
            .insert(goal.clone(), new_q_value);

        Ok(())
    }

    fn execute_gather_goal(&mut self, world: &mut World, spatial_map: &HashMap<(u32, u32), Vec<Entity>>, entity: Entity, resource_name: &str, _current_episode: u32) -> Result<(), SimulationError> {
        let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) else {
            return Ok(());
        };

        // 1. Prioritize known locations from memory
        if let Some(known_positions) = self.known_resources.get(resource_name) {
            let mut sorted_positions: Vec<_> = known_positions.iter().collect();
            sorted_positions.sort_by_key(|pos| pos.x.abs_diff(player_pos.x) + pos.y.abs_diff(player_pos.y));

            for target_pos in sorted_positions {
                if let Some(entities_at_pos) = spatial_map.get(&(target_pos.x, target_pos.y)) {
                    for &target_entity in entities_at_pos {
                        if let Some(resource) = world.get_component::<super::components::Resource>(target_entity) {
                            if resource.name == resource_name {
                                let dx = (player_pos.x as i32 - target_pos.x as i32).abs();
                                let dy = (player_pos.y as i32 - target_pos.y as i32).abs();

                                if dx <= 1 && dy <= 1 {
                                    world.add_component(entity, WantsToGather { target: target_entity });
                                    return Ok(());
                                } else if self.current_path.is_none() {
                                    if let Some(path) = pathfinding::find_path((player_pos.x, player_pos.y), (target_pos.x, target_pos.y), &self.mental_map) {
                                        self.current_path = Some(path);
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Fallback to spiral search if no known resources are reachable or none are known
        for radius in 0..5 { // Limit search radius
            for i in -(radius as i32)..=(radius as i32) {
                for j in -(radius as i32)..=(radius as i32) {
                    if i.abs() != radius as i32 && j.abs() != radius as i32 {
                        continue;
                    }
                    let search_x = (player_pos.x as i32 + i) as u32;
                    let search_y = (player_pos.y as i32 + j) as u32;

                    if let Some(entities_at_pos) = spatial_map.get(&(search_x, search_y)) {
                        for &target_entity in entities_at_pos {
                            if let Some(resource) = world.get_component::<super::components::Resource>(target_entity) {
                                if resource.name == resource_name {
                                    if let Some(target_pos) = world.get_component::<Position>(target_entity).map(|p| *p) {
                                        let dx = (player_pos.x as i32 - target_pos.x as i32).abs();
                                        let dy = (player_pos.y as i32 - target_pos.y as i32).abs();

                                        if dx <= 1 && dy <= 1 {
                                            world.add_component(entity, WantsToGather { target: target_entity });
                                            return Ok(());
                                        } else if self.current_path.is_none() {
                                            if let Some(path) = pathfinding::find_path((player_pos.x, player_pos.y), (target_pos.x, target_pos.y), &self.mental_map) {
                                                self.current_path = Some(path);
                                                return Ok(());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. If still no target, the goal is currently impossible
        self.current_goal = None; // Let the planner decide what to do next (e.g., explore)
        Ok(())
    }

    fn execute_craft_item_goal(&mut self, world: &mut World, entity: Entity, item_name: &str, _current_episode: u32) -> Result<(), SimulationError> {
        // The planner should ensure we have the resources. Just craft.
        world.add_component(entity, WantsToCraft { item_name: item_name.to_string() });
        Ok(())
    }

    fn execute_build_goal(&mut self, world: &mut World, entity: Entity, structure_name: &str, _current_episode: u32) -> Result<(), SimulationError> {
        world.add_component(entity, WantsToBuild { structure_name: structure_name.to_string() });
        Ok(())
    }


    fn handle_threats(&mut self, world: &World, entity: Entity) -> bool {
        let Some(health) = world.get_component::<crate::components::Health>(entity) else { return false; };
        let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) else { return false; };

        let hostile_players: Vec<_> = self.player_memories.iter()
            .filter(|(_, mem)| mem.relationship == RelationshipStatus::Hostile)
            .collect();

        if !hostile_players.is_empty() {
            // Flee if outnumbered or low on health
            if hostile_players.len() > 1 || health.current < health.max / 2 {
                self.current_goal = Some(Goal::Flee);
                self.current_path = None; // Clear any existing path
                return true;
            } else {
                // Fight if conditions are favorable
                let mut closest_threat = None;
                let mut min_dist = u32::MAX;

                for (&id, _) in hostile_players {
                    if let Some(threat_pos) = world.get_component::<Position>(id as usize) {
                        let dist = player_pos.x.abs_diff(threat_pos.x) + player_pos.y.abs_diff(threat_pos.y);
                        if dist < min_dist {
                            min_dist = dist;
                            closest_threat = Some(id);
                        }
                    }
                }

                if let Some(id) = closest_threat {
                    self.current_goal = Some(Goal::Attack(id));
                    self.current_path = None;
                    return true;
                }
            }
        }
        false
    }

    fn execute_attack_goal(&mut self, world: &mut World, entity: Entity, target_id: u32, _current_episode: u32) -> Result<(), SimulationError> {
        // Simple attack logic: add WantsToAttack component
        world.add_component(entity, crate::components::WantsToAttack { target: target_id as usize });
        Ok(())
    }

    fn execute_flee_goal(&mut self, world: &mut World, entity: Entity, _current_episode: u32) -> Result<(), SimulationError> {
        let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) else {
            return Ok(());
        };

        let hostile_positions: Vec<_> = self.player_memories.iter()
            .filter(|(_, mem)| mem.relationship == RelationshipStatus::Hostile)
            .filter_map(|(&id, _)| world.get_component::<Position>(id as usize).cloned())
            .collect();

        if hostile_positions.is_empty() {
            // No threats in memory, stop fleeing.
            self.current_goal = None;
            return Ok(());
        }

        let avg_threat_x = hostile_positions.iter().map(|p| p.x as f32).sum::<f32>() / hostile_positions.len() as f32;
        let avg_threat_y = hostile_positions.iter().map(|p| p.y as f32).sum::<f32>() / hostile_positions.len() as f32;

        let flee_vec_x = player_pos.x as f32 - avg_threat_x;
        let flee_vec_y = player_pos.y as f32 - avg_threat_y;

        let norm = (flee_vec_x.powi(2) + flee_vec_y.powi(2)).sqrt();
        let flee_dx = if norm > 0.0 { (flee_vec_x / norm).round() as i32 } else { 0 };
        let flee_dy = if norm > 0.0 { (flee_vec_y / norm).round() as i32 } else { 0 };

        world.add_component(entity, crate::components::Velocity { dx: flee_dx, dy: flee_dy });

        Ok(())
    }

    fn execute_explore_goal(&mut self, world: &mut World, entity: Entity, _current_episode: u32) -> Result<(), SimulationError> {
        if self.current_path.is_some() {
            // Already exploring, let follow_path do its thing
            return Ok(());
        }

        let mut unvisited = Vec::new();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if self.mental_map[y as usize][x as usize].is_none() {
                    unvisited.push((x, y));
                }
            }
        }

        if !unvisited.is_empty() {
            let target_idx = rand::thread_rng().gen_range(0..unvisited.len());
            let target_pos = unvisited[target_idx];

            if let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) {
                if let Some(path) = pathfinding::find_path((player_pos.x, player_pos.y), target_pos, &self.mental_map) {
                    self.current_path = Some(path);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;
    use crate::recipes::RecipeManager;
    use crate::item::{ItemRegistry, Item};
    use crate::player::Player;
    use crate::components::Resource;
    use std::sync::Arc;
    use std::env;

    fn create_test_brain() -> Brain {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let recipe_manager = Arc::new(RecipeManager::new(&format!("{}/recipes.json", manifest_dir)));
        Brain::new(recipe_manager, 0.1, 0.9, 0.1)
    }

    #[test]
    fn test_choose_goal_randomly() {
        let brain = create_test_brain();
        let world = World::new();
        let state = HighLevelState {
            has_wood: false,
            has_stone: false,
            has_iron_ore: false,
            has_stone_axe: false,
            num_hostile_players: 0,
            health_level: 100,
            is_night: false,
        };
        let goal = brain.choose_goal(&world, &state).unwrap();
        assert!(brain.goals.contains(&goal));
    }

    #[test]
    fn test_choose_goal_q_learning() {
        let mut brain = create_test_brain();
        let mut world = World::new();
        brain.epsilon = 0.0; // Ensure Q-table is used

        let resource_entity = world.create_entity();
        world.add_component(resource_entity, Resource { name: "stone".to_string(), quantity: 5 });
        world.add_component(resource_entity, Position { x: 1, y: 1 });
        brain.known_resources.entry("stone".to_string()).or_default().insert(Position { x: 1, y: 1 });


        let state = HighLevelState {
            has_wood: true,
            has_stone: false,
            has_iron_ore: false,
            has_stone_axe: false,
            num_hostile_players: 0,
            health_level: 100,
            is_night: false,
        };
        let state_key = serde_json::to_string(&state).unwrap();
        let mut q_values = HashMap::new();
        q_values.insert(Goal::GatherResource("stone".to_string()), 10.0);
        q_values.insert(Goal::CraftItem("stone_axe".to_string()), 5.0);
        brain.goal_q_table.insert(state_key, q_values);

        let chosen_goal = brain.choose_goal(&world, &state).unwrap();
        assert_eq!(chosen_goal, Goal::GatherResource("stone".to_string()));
    }

    #[test]
    fn test_is_goal_complete() {
        let brain = create_test_brain();
        let mut world = World::new();
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let item_registry = ItemRegistry::new(&format!("{}/items.json", manifest_dir));


        let player_entity = world.create_entity();
        let player = Player::new(0, 100, 100);
        world.add_component(player_entity, player);

        let mut inventory = Inventory::new();
        inventory.add_item("wood", 11);
        world.add_component(player_entity, inventory);

        let goal = Goal::GatherResource("wood".to_string());
        assert!(brain.is_goal_complete(&world, player_entity, &goal));

        let goal = Goal::GatherResource("stone".to_string());
        assert!(!brain.is_goal_complete(&world, player_entity, &goal));
    }

    #[test]
    fn test_mental_map() {
        let mut brain = create_test_brain();
        let mut world = World::new();
        let mut spatial_map = HashMap::new();

        let player_entity = world.create_entity();
        world.add_component(player_entity, Position { x: 5, y: 5 });

        let visible_tiles = vec![
            (Position { x: 5, y: 5 }, Tile::new('.', "plains".to_string())),
            (Position { x: 5, y: 6 }, Tile::new('T', "forest".to_string())),
        ];

        let resource_entity = world.create_entity();
        world.add_component(resource_entity, Resource { name: "wood".to_string(), quantity: 5 });
        world.add_component(resource_entity, Position { x: 5, y: 6 });
        spatial_map.insert((5, 6), vec![resource_entity]);

        brain.update_mental_map(&visible_tiles, &world, &spatial_map);

        assert!(brain.mental_map[5][5].is_some());
        assert_eq!(brain.mental_map[5][5].as_ref().unwrap().tile.tile_type, '.');
        assert!(brain.mental_map[6][5].is_some());
        assert_eq!(brain.mental_map[6][5].as_ref().unwrap().tile.tile_type, 'T');

        let goal = Goal::GatherResource("wood".to_string());
        assert!(brain.is_goal_valid(&world, &goal));

        let goal = Goal::GatherResource("stone".to_string());
        assert!(!brain.is_goal_valid(&world, &goal));
    }

    #[test]
    fn test_update_q_table() {
        let mut brain = create_test_brain();
        let prev_state = HighLevelState {
            has_wood: false,
            has_stone: false,
            has_iron_ore: false,
            has_stone_axe: false,
            num_hostile_players: 0,
            health_level: 100,
            is_night: false,
        };
        let goal = Goal::GatherResource("wood".to_string());
        let reward = 10.0;
        let new_state = HighLevelState {
            has_wood: true,
            has_stone: false,
            has_iron_ore: false,
            has_stone_axe: false,
            num_hostile_players: 0,
            health_level: 100,
            is_night: false,
        };

        brain.update_q_table(&prev_state, &goal, reward, &new_state).unwrap();

        let prev_state_key = serde_json::to_string(&prev_state).unwrap();
        let q_value = brain.goal_q_table.get(&prev_state_key).unwrap().get(&goal).unwrap();
        assert_eq!(*q_value, 1.0); // 0 + 0.1 * (10 + 0.9 * 0 - 0) = 1.0
    }

    #[test]
    fn test_multi_step_planning() {
        // 1. Setup
        let mut recipes = HashMap::new();
        recipes.insert("plank".to_string(), {
            let mut ingredients = HashMap::new();
            ingredients.insert("wood".to_string(), 1);
            ingredients
        });
        recipes.insert("wooden_box".to_string(), {
            let mut ingredients = HashMap::new();
            ingredients.insert("plank".to_string(), 4);
            ingredients
        });
        let recipe_manager = Arc::new(RecipeManager::with_recipes(recipes));
        let brain = Brain::new(recipe_manager, 0.1, 0.9, 0.1);

        let mut world = World::new();
        let player_entity = world.create_entity();
        world.add_component(player_entity, Inventory::new());

        // 2. Plan
        let goal = Goal::CraftItem("wooden_box".to_string());
        let plan = brain.plan_goal(&world, player_entity, &goal).unwrap();

        // 3. Assert
        let expected_plan = vec![
            Goal::Explore,
            Goal::GatherResource("wood".to_string()),
            Goal::CraftItem("wooden_box".to_string()),
        ];

        let plan_set: std::collections::HashSet<_> = plan.into_iter().collect();
        let expected_set: std::collections::HashSet<_> = expected_plan.into_iter().collect();

        assert_eq!(plan_set, expected_set);
    }

    #[test]
    fn test_planning_with_unknown_resource_triggers_explore() {
        // 1. Setup
        let mut recipes = HashMap::new();
        recipes.insert("iron_pickaxe".to_string(), {
            let mut ingredients = HashMap::new();
            ingredients.insert("iron_ore".to_string(), 3);
            ingredients
        });
        let recipe_manager = Arc::new(RecipeManager::with_recipes(recipes));
        let mut brain = Brain::new(recipe_manager, 0.1, 0.9, 0.1);
        brain.known_resources.remove("iron_ore"); // Ensure it's unknown

        let mut world = World::new();
        let player_entity = world.create_entity();
        world.add_component(player_entity, Inventory::new());

        // 2. Plan
        let goal = Goal::CraftItem("iron_pickaxe".to_string());
        let plan = brain.plan_goal(&world, player_entity, &goal).unwrap();

        // 3. Assert
        let expected_plan = vec![
            Goal::Explore,
            Goal::GatherResource("iron_ore".to_string()),
            Goal::CraftItem("iron_pickaxe".to_string()),
        ];
        assert_eq!(plan, expected_plan);
    }

    #[test]
    fn test_planning_with_known_resource() {
        // 1. Setup
        let mut recipes = HashMap::new();
        recipes.insert("stone_axe".to_string(), {
            let mut ingredients = HashMap::new();
            ingredients.insert("stone".to_string(), 2);
            ingredients
        });
        let recipe_manager = Arc::new(RecipeManager::with_recipes(recipes));
        let mut brain = Brain::new(recipe_manager, 0.1, 0.9, 0.1);
        // Ensure the brain DOES know where stone is
        brain.known_resources.entry("stone".to_string()).or_default().insert(Position { x: 10, y: 10 });

        let mut world = World::new();
        let player_entity = world.create_entity();
        world.add_component(player_entity, Inventory::new());

        // 2. Plan
        let goal = Goal::CraftItem("stone_axe".to_string());
        let plan = brain.plan_goal(&world, player_entity, &goal).unwrap();

        // 3. Assert
        let expected_plan = vec![
            Goal::GatherResource("stone".to_string()),
            Goal::CraftItem("stone_axe".to_string()),
        ];
        assert_eq!(plan, expected_plan);
    }

    #[test]
    fn test_threat_assessment_outnumbered() {
        let mut brain = create_test_brain();
        let mut world = World::new();

        let player_entity = world.create_entity();
        world.add_component(player_entity, crate::components::Health { current: 100, max: 100 });
        world.add_component(player_entity, Position { x: 5, y: 5 });

        // Two hostile players
        brain.player_memories.insert(1, PlayerMemory { relationship: RelationshipStatus::Hostile });
        brain.player_memories.insert(2, PlayerMemory { relationship: RelationshipStatus::Hostile });

        let handled = brain.handle_threats(&world, player_entity);

        assert!(handled);
        assert_eq!(brain.current_goal, Some(Goal::Flee));
    }

    #[test]
    fn test_opportunistic_gathering() {
        let mut brain = create_test_brain();
        let mut world = World::new();
        let mut spatial_map = HashMap::new();

        let player_entity = world.create_entity();
        world.add_component(player_entity, Inventory::new());
        world.add_component(player_entity, Position { x: 5, y: 5 });

        // Give the AI a low-commitment goal
        brain.current_goal = Some(Goal::GatherResource("wood".to_string()));
        brain.goal_commitment_ticks = 2;

        // A valuable resource appears in visible tiles
        let iron_ore_entity = world.create_entity();
        world.add_component(iron_ore_entity, Resource { name: "iron_ore".to_string(), quantity: 1 });
        world.add_component(iron_ore_entity, Position { x: 5, y: 6 });
        spatial_map.insert((5, 6), vec![iron_ore_entity]);
        let visible_tiles = vec![(Position { x: 5, y: 6 }, Tile::new('.', "plains".to_string()))];

        brain.handle_opportunities(&world, player_entity, &spatial_map, &visible_tiles);

        assert_eq!(brain.current_goal, Some(Goal::GatherResource("iron_ore".to_string())));
    }

    #[test]
    fn test_night_behavior_prefers_building() {
        let mut brain = create_test_brain();
        let world = World::new();

        // State where it's night
        let night_state = HighLevelState {
            has_wood: true, // Has resources to build
            has_stone: true,
            has_iron_ore: false,
            has_stone_axe: false,
            num_hostile_players: 0,
            health_level: 100,
            is_night: true,
        };

        // Make sure building is a valid goal
        brain.goals.push(Goal::Build("foundation".to_string()));

        // Set Q-values to make other goals seem better
        let state_key = serde_json::to_string(&night_state).unwrap();
        let mut q_values = HashMap::new();
        q_values.insert(Goal::GatherResource("wood".to_string()), 20.0); // High value for another goal
        q_values.insert(Goal::Build("foundation".to_string()), 5.0); // Lower value for building
        brain.goal_q_table.insert(state_key, q_values);
        brain.epsilon = 0.0; // Ensure we use Q-values

        let chosen_goal = brain.choose_goal(&world, &night_state).unwrap();

        // Assert that building was chosen despite the lower Q-value, due to the night bonus
        assert_eq!(chosen_goal, Goal::Build("foundation".to_string()));
    }
}
