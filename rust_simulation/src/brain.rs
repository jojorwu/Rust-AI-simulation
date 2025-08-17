use rand::Rng;
use super::errors::SimulationError;
use super::config::{WIDTH, HEIGHT};
use std::cmp::Ordering;
use super::map::Tile;
use super::pathfinding;
use crate::components::{Position, WantsToGather, WantsToCraft, WantsToBuild, Resource};
use std::collections::HashMap;
use super::recipes::RecipeManager;
use super::ecs::{World, Entity};
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
    pub fn new(recipe_manager: Arc<RecipeManager>, epsilon: f64) -> Self {
        let goals = vec![
            Goal::GatherResource("wood".to_string()),
            Goal::GatherResource("stone".to_string()),
            Goal::CraftItem("stone_axe".to_string()),
            Goal::Build("foundation".to_string()),
        ];
        Brain {
            goals,
            recipe_manager,
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
            q_values.iter()
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
        if let Some(player) = world.get_component::<Player>(entity) {
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
        } else {
            false
        }
    }

    pub fn tick(&mut self, world: &mut World, spatial_map: &HashMap<(u32, u32), Vec<Entity>>, entity: Entity, high_level_state: &HighLevelState, visible_tiles: &Vec<(Position, Tile)>) -> Result<(), SimulationError> {
        self.update_mental_map(visible_tiles);
        self.update_current_goal(world, entity, high_level_state)?;
        self.choose_action_for_goal(world, spatial_map, entity, 0) // episode is not used anymore
    }

    fn update_mental_map(&mut self, visible_tiles: &Vec<(Position, Tile)>) {
        for (pos, tile) in visible_tiles {
            self.mental_map[pos.y as usize][pos.x as usize] = Some(MemoryTile {
                tile: tile.clone(),
            });
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
            if self.is_goal_complete(world, entity, goal) || !self.is_goal_valid(world, goal) {
                self.current_goal = None;
                self.current_path = None;
                self.goal_commitment_ticks = 0;
            }
        }

        if self.current_goal.is_none() && self.goal_commitment_ticks == 0 {
            self.current_goal = Some(self.choose_goal(world, high_level_state)?);
            self.goal_commitment_ticks = 10; // Commit to the new goal for 10 ticks
        }
        Ok(())
    }

    fn is_goal_valid(&self, world: &World, goal: &Goal) -> bool {
        match goal {
            Goal::GatherResource(resource_name) => {
                for y in 0..self.mental_map.len() {
                    for x in 0..self.mental_map[y].len() {
                        if self.mental_map[y][x].is_some() {
                            // This is not efficient, but it's a start.
                            // A better approach would be to store resource locations in the mental map.
                            for entity in 0..world.entities.len() {
                                if let (Some(pos), Some(res)) = (
                                    world.get_component::<Position>(entity),
                                    world.get_component::<Resource>(entity),
                                ) {
                                    if pos.x == x as u32 && pos.y == y as u32 && res.name == *resource_name {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
                false
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

    fn execute_gather_goal(&mut self, world: &mut World, spatial_map: &HashMap<(u32, u32), Vec<Entity>>, entity: Entity, resource_name: &str, _current_episode: u32) -> Result<(), SimulationError> {
        let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) else {
            return Ok(());
        };

        // Find the closest resource using the spatial map
        let mut closest_target = None;
        let mut min_dist = u32::MAX;

        // Search in a spiral pattern around the player
        for radius in 0..10 { // search radius
            for i in -(radius as i32)..=(radius as i32) {
                for j in -(radius as i32)..=(radius as i32) {
                    if i.abs() != radius as i32 && j.abs() != radius as i32 {
                        continue; // only check the perimeter of the search box
                    }
                    let search_x = (player_pos.x as i32 + i) as u32;
                    let search_y = (player_pos.y as i32 + j) as u32;

                    if let Some(entities_at_pos) = spatial_map.get(&(search_x, search_y)) {
                        for &target_entity in entities_at_pos {
                            if let Some(resource) = world.get_component::<super::components::Resource>(target_entity) {
                                if resource.name == resource_name {
                                    let dist = radius as u32;
                                    if dist < min_dist {
                                        min_dist = dist;
                                        closest_target = Some(target_entity);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if closest_target.is_some() {
                break; // Found a target in this radius, no need to search further
            }
        }

        if let Some(target) = closest_target {
            if let Some(target_pos) = world.get_component::<Position>(target).map(|p| *p) {
                let dx = (player_pos.x as i32 - target_pos.x as i32).abs();
                let dy = (player_pos.y as i32 - target_pos.y as i32).abs();

                if dx <= 1 && dy <= 1 {
                    world.add_component(entity, WantsToGather { target });
                } else if self.current_path.is_none() {
                    if let Some(path) = pathfinding::find_path((player_pos.x, player_pos.y), (target_pos.x, target_pos.y), &self.mental_map) {
                        self.current_path = Some(path);
                    }
                }
            }
        } else {
            // If we reach here, it means we couldn't find a path or the resource doesn't exist.
            // Clear the goal to avoid getting stuck.
            self.current_goal = None;
        }

        Ok(())
    }

    fn execute_craft_item_goal(&mut self, world: &mut World, entity: Entity, item_name: &str, _current_episode: u32) -> Result<(), SimulationError> {
        let recipe = self.recipe_manager.get_required_resources(item_name, 1);
        let mut missing_resource = None;
        let Some(player) = world.get_component::<Player>(entity) else {
            return Ok(());
        };

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
        } else {
            // We have all the resources, so we can craft the item.
            self.current_goal = self.goal_stack.pop(); // Go back to the parent goal
            world.add_component(entity, WantsToCraft { item_name: item_name.to_string() });
        }
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
            if health.current < health.max / 2 {
                self.current_goal = Some(Goal::Flee);
                self.current_path = None; // Clear any existing path
                return true;
            } else {
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

    fn create_test_brain() -> Brain {
        let recipe_manager = Arc::new(RecipeManager::new("recipes.json"));
        Brain::new(recipe_manager, 0.1)
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
        brain.mental_map[1][1] = Some(MemoryTile {
            tile: Tile::new('s', "mountains".to_string()),
        });

        let state = HighLevelState {
            has_wood: true,
            has_stone: false,
            has_iron_ore: false,
            has_stone_axe: false,
            num_hostile_players: 0,
            health_level: 100,
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
        let mut item_registry = ItemRegistry::new("items.json");
        item_registry.items.insert("wood".to_string(), Item { name: "wood".to_string(), stackable: true, tool: false, properties: None });

        let player_entity = world.create_entity();
        let mut player = Player::new(0);
        player.add_item("wood", 11, None, &item_registry);
        world.add_component(player_entity, player);

        let goal = Goal::GatherResource("wood".to_string());
        assert!(brain.is_goal_complete(&world, player_entity, &goal));

        let goal = Goal::GatherResource("stone".to_string());
        assert!(!brain.is_goal_complete(&world, player_entity, &goal));
    }

    #[test]
    fn test_mental_map() {
        let mut brain = create_test_brain();
        let mut world = World::new();
        let player_entity = world.create_entity();
        world.add_component(player_entity, Position { x: 5, y: 5 });

        let visible_tiles = vec![
            (Position { x: 5, y: 5 }, Tile::new('.', "plains".to_string())),
            (Position { x: 5, y: 6 }, Tile::new('T', "forest".to_string())),
        ];

        let resource_entity = world.create_entity();
        world.add_component(resource_entity, Resource { name: "wood".to_string(), quantity: 5 });
        world.add_component(resource_entity, Position { x: 5, y: 6 });

        brain.update_mental_map(&visible_tiles);

        assert!(brain.mental_map[5][5].is_some());
        assert_eq!(brain.mental_map[5][5].as_ref().unwrap().tile.tile_type, '.');
        assert!(brain.mental_map[6][5].is_some());
        assert_eq!(brain.mental_map[6][5].as_ref().unwrap().tile.tile_type, 'T');

        let goal = Goal::GatherResource("wood".to_string());
        assert!(brain.is_goal_valid(&world, &goal));

        let goal = Goal::GatherResource("stone".to_string());
        assert!(!brain.is_goal_valid(&world, &goal));
    }
}
