use super::config::{
    BUILD_GOAL_BONUS, GATHER_GOAL_THRESHOLD, GOAL_COMMITMENT_TICKS, GOAL_PENALTY, GOAL_REWARD,
    HEIGHT, THREAT_GOAL_COMMITMENT_TICKS, WIDTH,
};
use super::ecs::{Entity, World};
use super::errors::SimulationError;
use super::map::Tile;
use super::pathfinding;
use super::recipes::RecipeManager;
use crate::components::{
    BrainComponent, Chest, Health, Inventory, Position, Resource, Velocity, WantsToBuild,
    WantsToCraft, WantsToGather, WantsToStoreItem,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Goal {
    GatherResource(String),
    CraftItem(String),
    Build(String),
    Attack(u32),
    Flee,
    Explore,
    Stockpile(String),
}

#[derive(Debug)]
pub enum BrainAction {
    Move(Velocity),
    Gather(WantsToGather),
    Craft(WantsToCraft),
    Build(WantsToBuild),
    Attack(crate::components::WantsToAttack),
    Store(WantsToStoreItem),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySummary {
    pub has_wood: bool,
    pub has_stone: bool,
    pub has_iron_ore: bool,
    pub has_stone_axe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighLevelState {
    pub inventory_summary: InventorySummary,
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

/// The Brain struct is now a stateless logic processor for the AI.
pub struct Brain {
    pub goals: Vec<Goal>,
    pub recipe_manager: Arc<RecipeManager>,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub epsilon: f64,
}

impl Brain {
    pub fn new(
        recipe_manager: Arc<RecipeManager>,
        learning_rate: f64,
        discount_factor: f64,
        epsilon: f64,
    ) -> Self {
        let goals = vec![
            Goal::GatherResource("wood".to_string()),
            Goal::GatherResource("stone".to_string()),
            Goal::CraftItem("stone_axe".to_string()),
            Goal::Build("foundation".to_string()),
            Goal::Stockpile("wood".to_string()),
        ];
        Brain {
            goals,
            recipe_manager,
            learning_rate,
            discount_factor,
            epsilon,
        }
    }

    pub fn save_q_table(&self, brain_component: &BrainComponent) -> Result<(), SimulationError> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let q_table_path = std::path::Path::new(manifest_dir).join("../q_table.json");
        let json = serde_json::to_string_pretty(&brain_component.goal_q_table)?;
        fs::write(q_table_path, json)?;
        Ok(())
    }

    pub fn choose_goal(
        &self,
        brain_component: &BrainComponent,
        world: &World,
        state: &HighLevelState,
    ) -> Result<Goal, SimulationError> {
        let valid_goals: Vec<_> = self
            .goals
            .iter()
            .filter(|g| self.is_goal_valid(brain_component, world, g))
            .cloned()
            .collect();
        if valid_goals.is_empty() {
            return Ok(Goal::Flee);
        }

        let choose_random_goal = || {
            let index = rand::thread_rng().gen_range(0..valid_goals.len());
            Ok(valid_goals[index].clone())
        };

        if rand::thread_rng().r#gen::<f64>() < self.epsilon {
            return choose_random_goal();
        }

        let state_key_str = serde_json::to_string(state)?;
        if let Some(q_values) = brain_component.goal_q_table.get(&state_key_str) {
            let mut modified_q_values = q_values.clone();
            if state.is_night {
                for (goal, q_value) in modified_q_values.iter_mut() {
                    if let Goal::Build(_) = goal {
                        *q_value += BUILD_GOAL_BONUS;
                    }
                }
            }

            modified_q_values
                .iter()
                .filter(|(g, _)| self.is_goal_valid(brain_component, world, g))
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(goal, _)| goal.clone())
                .map(Ok)
                .unwrap_or_else(choose_random_goal)
        } else {
            choose_random_goal()
        }
    }

    pub fn is_goal_complete(
        &self,
        brain_component: &BrainComponent,
        world: &World,
        entity: Entity,
        goal: &Goal,
    ) -> bool {
        if let Some(inventory) = world.get_component::<Inventory>(entity) {
            match goal {
                Goal::GatherResource(resource) => {
                    if let Some(parent_goal) = brain_component.goal_stack.last() {
                        if let Goal::CraftItem(item_name) = parent_goal {
                            let recipe = self.recipe_manager.get_required_resources(item_name, 1);
                            if let Some(&required_amount) = recipe.get(resource) {
                                return inventory.get_quantity(resource) >= required_amount;
                            }
                        }
                    }
                    inventory.get_quantity(resource) > GATHER_GOAL_THRESHOLD
                }
                Goal::CraftItem(item) => inventory.has_item(item, 1),
                Goal::Explore => brain_component
                    .current_path
                    .as_ref()
                    .map_or(true, |p| p.is_empty()),
                Goal::Stockpile(resource) => !inventory.has_item(resource, 1),
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn plan_goal(
        &self,
        brain_component: &BrainComponent,
        world: &World,
        entity: Entity,
        goal: &Goal,
    ) -> Result<Vec<Goal>, SimulationError> {
        let mut plan = Vec::new();
        match goal {
            Goal::CraftItem(item_name) => {
                let required = self.recipe_manager.get_required_resources(item_name, 1);
                let inventory = world.get_component::<Inventory>(entity);
                plan.extend(self.plan_resource_gathering(brain_component, inventory, &required));
                plan.push(goal.clone());
            }
            Goal::Build(structure_name) => {
                let required = self
                    .recipe_manager
                    .get_required_resources(structure_name, 1);
                let inventory = world.get_component::<Inventory>(entity);
                plan.extend(self.plan_resource_gathering(brain_component, inventory, &required));
                plan.push(goal.clone());
            }
            Goal::Stockpile(resource) => {
                let inventory = world.get_component::<Inventory>(entity);
                let has_enough = inventory.map_or(false, |inv| inv.has_item(resource, 1));
                if !has_enough {
                    plan.push(Goal::GatherResource(resource.clone()));
                }
                plan.push(goal.clone());
            }
            _ => {
                plan.push(goal.clone());
            }
        }
        Ok(plan)
    }

    fn plan_resource_gathering(
        &self,
        brain_component: &BrainComponent,
        inventory: Option<&Inventory>,
        required: &HashMap<String, u32>,
    ) -> Vec<Goal> {
        let mut plan = Vec::new();
        for (resource, &required_amount) in required {
            let has_enough =
                inventory.map_or(false, |inv| inv.get_quantity(resource) >= required_amount);
            if !has_enough {
                if !brain_component.known_resources.contains_key(resource) {
                    plan.push(Goal::Explore);
                }
                plan.push(Goal::GatherResource(resource.clone()));
            }
        }
        plan
    }

    pub fn tick(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        spatial_map: &HashMap<(u32, u32), Vec<Entity>>,
        entity: Entity,
        high_level_state: &HighLevelState,
        visible_tiles: &Vec<(Position, Tile)>,
    ) -> Result<Option<BrainAction>, SimulationError> {
        self.update_q_table_based_on_previous_action(
            brain_component,
            world,
            entity,
            high_level_state,
        )?;

        self.update_internal_state(brain_component, world, entity, spatial_map, visible_tiles);
        self.update_goal_and_plan(brain_component, world, entity, high_level_state)?;

        let action =
            self.choose_and_execute_action(brain_component, world, spatial_map, entity, 0)?;

        brain_component.prev_state = Some(high_level_state.clone());
        brain_component.prev_goal = brain_component.current_goal.clone();

        Ok(action)
    }

    fn update_q_table_based_on_previous_action(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        entity: Entity,
        high_level_state: &HighLevelState,
    ) -> Result<(), SimulationError> {
        if let (Some(prev_state), Some(prev_goal)) = (
            brain_component.prev_state.clone(),
            brain_component.prev_goal.clone(),
        ) {
            let reward = if self.is_goal_complete(brain_component, world, entity, &prev_goal) {
                GOAL_REWARD
            } else {
                GOAL_PENALTY
            };
            self._update_q_table(
                brain_component,
                &prev_state,
                &prev_goal,
                reward,
                high_level_state,
            )?;
        }
        Ok(())
    }

    fn update_internal_state(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        entity: Entity,
        spatial_map: &HashMap<(u32, u32), Vec<Entity>>,
        visible_tiles: &Vec<(Position, Tile)>,
    ) {
        self.update_mental_map(brain_component, world, spatial_map, visible_tiles);
        self.handle_opportunities(brain_component, world, entity, spatial_map, visible_tiles);
    }

    fn handle_opportunities(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        entity: Entity,
        spatial_map: &HashMap<(u32, u32), Vec<Entity>>,
        visible_tiles: &Vec<(Position, Tile)>,
    ) {
        if brain_component.goal_commitment_ticks
            >= crate::config::OPPORTUNISTIC_COMMITMENT_THRESHOLD
        {
            return;
        }
        for (pos, _tile) in visible_tiles {
            if let Some(entities_at_pos) = spatial_map.get(&(pos.x, pos.y)) {
                for &entity_id in entities_at_pos {
                    if let Some(resource) = world.get_component::<Resource>(entity_id) {
                        if crate::config::VALUABLE_RESOURCES.contains(&resource.name.as_str()) {
                            let has_it_already = world
                                .get_component::<Inventory>(entity)
                                .map_or(false, |inv| inv.get_quantity(&resource.name) > 0);
                            if !has_it_already {
                                brain_component.goal_stack.clear();
                                brain_component.current_path = None;
                                brain_component.current_goal =
                                    Some(Goal::GatherResource(resource.name.clone()));
                                brain_component.goal_commitment_ticks = GOAL_COMMITMENT_TICKS;
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_mental_map(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        spatial_map: &HashMap<(u32, u32), Vec<Entity>>,
        visible_tiles: &Vec<(Position, Tile)>,
    ) {
        for (pos, tile) in visible_tiles {
            brain_component.mental_map[pos.y as usize][pos.x as usize] =
                Some(MemoryTile { tile: tile.clone() });
            if let Some(entities_at_pos) = spatial_map.get(&(pos.x, pos.y)) {
                for &entity_id in entities_at_pos {
                    if let Some(resource) = world.get_component::<Resource>(entity_id) {
                        brain_component
                            .known_resources
                            .entry(resource.name.clone())
                            .or_default()
                            .insert(*pos);
                    }
                }
            }
        }
    }

    fn update_goal_and_plan(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        entity: Entity,
        high_level_state: &HighLevelState,
    ) -> Result<(), SimulationError> {
        self._update_current_goal(brain_component, world, entity, high_level_state)
    }

    fn _update_current_goal(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        entity: Entity,
        high_level_state: &HighLevelState,
    ) -> Result<(), SimulationError> {
        if self.handle_threats(brain_component, world, entity) {
            brain_component.goal_commitment_ticks = THREAT_GOAL_COMMITMENT_TICKS;
            return Ok(());
        }

        if brain_component.goal_commitment_ticks > 0 {
            brain_component.goal_commitment_ticks -= 1;
        }

        if let Some(goal) = &brain_component.current_goal {
            if self.is_goal_complete(brain_component, world, entity, goal) {
                brain_component.current_path = None;
                brain_component.current_goal = brain_component.goal_stack.pop();
            } else if !self.is_goal_valid(brain_component, world, goal) {
                brain_component.current_goal = None;
                brain_component.goal_stack.clear();
                brain_component.current_path = None;
                brain_component.goal_commitment_ticks = 0;
            }
        }

        if brain_component.current_goal.is_none() && brain_component.goal_commitment_ticks == 0 {
            let new_high_level_goal = self.choose_goal(brain_component, world, high_level_state)?;
            let mut plan = self.plan_goal(brain_component, world, entity, &new_high_level_goal)?;
            plan.reverse();
            brain_component.goal_stack = plan;
            brain_component.current_goal = brain_component.goal_stack.pop();
            if brain_component.current_goal.is_some() {
                brain_component.goal_commitment_ticks = GOAL_COMMITMENT_TICKS;
            }
        }
        Ok(())
    }

    fn is_goal_valid(&self, brain_component: &BrainComponent, _world: &World, goal: &Goal) -> bool {
        match goal {
            Goal::GatherResource(resource_name) => brain_component
                .known_resources
                .get(resource_name)
                .map_or(false, |p| !p.is_empty()),
            _ => true,
        }
    }

    fn choose_and_execute_action(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        spatial_map: &HashMap<(u32, u32), Vec<Entity>>,
        entity: Entity,
        current_episode: u32,
    ) -> Result<Option<BrainAction>, SimulationError> {
        self._choose_action_for_goal(brain_component, world, spatial_map, entity, current_episode)
    }

    fn _choose_action_for_goal(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        spatial_map: &HashMap<(u32, u32), Vec<Entity>>,
        entity: Entity,
        current_episode: u32,
    ) -> Result<Option<BrainAction>, SimulationError> {
        if let Some(action) = self.follow_path(brain_component, world, entity) {
            return Ok(Some(action));
        }
        if let Some(goal) = brain_component.current_goal.clone() {
            match goal {
                Goal::GatherResource(name) => self.execute_gather_goal(
                    brain_component,
                    world,
                    spatial_map,
                    entity,
                    &name,
                    current_episode,
                ),
                Goal::CraftItem(name) => {
                    self.execute_craft_item_goal(entity, &name, current_episode)
                }
                Goal::Build(name) => self.execute_build_goal(entity, &name, current_episode),
                Goal::Attack(id) => self.execute_attack_goal(entity, id, current_episode),
                Goal::Flee => {
                    self.execute_flee_goal(brain_component, world, entity, current_episode)
                }
                Goal::Explore => {
                    self.execute_explore_goal(brain_component, world, entity, current_episode)
                }
                Goal::Stockpile(res) => self.execute_stockpile_goal(
                    brain_component,
                    world,
                    entity,
                    &res,
                    current_episode,
                ),
            }
        } else {
            Ok(None)
        }
    }

    fn follow_path(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        entity: Entity,
    ) -> Option<BrainAction> {
        if let Some(path) = &mut brain_component.current_path {
            if !path.is_empty() {
                if let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) {
                    let next_pos = path.remove(0);
                    let (dx, dy) = (
                        next_pos.0 as i32 - player_pos.x as i32,
                        next_pos.1 as i32 - player_pos.y as i32,
                    );
                    return Some(BrainAction::Move(crate::components::Velocity { dx, dy }));
                }
            } else {
                brain_component.current_path = None;
            }
        }
        None
    }

    fn _update_q_table(
        &self,
        brain_component: &mut BrainComponent,
        prev_state: &HighLevelState,
        goal: &Goal,
        reward: f64,
        new_state: &HighLevelState,
    ) -> Result<(), SimulationError> {
        let prev_state_key = serde_json::to_string(prev_state)?;
        let new_state_key = serde_json::to_string(new_state)?;
        let old_q_value = brain_component
            .goal_q_table
            .get(&prev_state_key)
            .and_then(|q| q.get(goal))
            .cloned()
            .unwrap_or(0.0);
        let max_future_q = brain_component
            .goal_q_table
            .get(&new_state_key)
            .map(|q| {
                q.values()
                    .cloned()
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or(0.0)
            })
            .unwrap_or(0.0);
        let new_q_value = old_q_value
            + self.learning_rate * (reward + self.discount_factor * max_future_q - old_q_value);
        brain_component
            .goal_q_table
            .entry(prev_state_key)
            .or_default()
            .insert(goal.clone(), new_q_value);
        Ok(())
    }

    fn execute_gather_goal(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        spatial_map: &HashMap<(u32, u32), Vec<Entity>>,
        entity: Entity,
        resource_name: &str,
        _current_episode: u32,
    ) -> Result<Option<BrainAction>, SimulationError> {
        let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) else {
            return Ok(None);
        };
        if let Some(known_positions) = brain_component.known_resources.get(resource_name) {
            let mut sorted_positions: Vec<_> = known_positions.iter().collect();
            sorted_positions
                .sort_by_key(|pos| pos.x.abs_diff(player_pos.x) + pos.y.abs_diff(player_pos.y));
            for target_pos in sorted_positions {
                if let Some(entities_at_pos) = spatial_map.get(&(target_pos.x, target_pos.y)) {
                    for &target_entity in entities_at_pos {
                        if let Some(resource) =
                            world.get_component::<super::components::Resource>(target_entity)
                        {
                            if resource.name == resource_name {
                                let (dx, dy) = (
                                    (player_pos.x as i32 - target_pos.x as i32).abs(),
                                    (player_pos.y as i32 - target_pos.y as i32).abs(),
                                );
                                if dx <= 1 && dy <= 1 {
                                    return Ok(Some(BrainAction::Gather(WantsToGather {
                                        target: target_entity,
                                    })));
                                } else if brain_component.current_path.is_none() {
                                    if let Some(path) = pathfinding::find_path(
                                        (player_pos.x, player_pos.y),
                                        (target_pos.x, target_pos.y),
                                        &brain_component.mental_map,
                                    ) {
                                        brain_component.current_path = Some(path);
                                        return Ok(None);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // Fallback to searching nearby if not found in known resources
        brain_component.current_goal = None;
        Ok(None)
    }

    fn execute_craft_item_goal(
        &self,
        _entity: Entity,
        item_name: &str,
        _current_episode: u32,
    ) -> Result<Option<BrainAction>, SimulationError> {
        Ok(Some(BrainAction::Craft(WantsToCraft {
            item_name: item_name.to_string(),
        })))
    }

    fn execute_build_goal(
        &self,
        _entity: Entity,
        structure_name: &str,
        _current_episode: u32,
    ) -> Result<Option<BrainAction>, SimulationError> {
        Ok(Some(BrainAction::Build(WantsToBuild {
            structure_name: structure_name.to_string(),
        })))
    }

    fn handle_threats(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        entity: Entity,
    ) -> bool {
        let Some(health) = world.get_component::<Health>(entity) else {
            return false;
        };
        let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) else {
            return false;
        };
        let hostile_players: Vec<_> = brain_component
            .player_memories
            .iter()
            .filter(|(_, mem)| mem.relationship == RelationshipStatus::Hostile)
            .map(|(id, _)| *id)
            .collect();
        if hostile_players.is_empty() {
            return false;
        }
        if self.handle_territorial_threats(
            brain_component,
            world,
            &player_pos,
            health,
            &hostile_players,
        ) {
            return true;
        }
        if self.handle_standard_threats(
            brain_component,
            world,
            &player_pos,
            health,
            &hostile_players,
        ) {
            return true;
        }
        false
    }

    fn handle_territorial_threats(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        player_pos: &Position,
        health: &Health,
        hostile_players: &[u32],
    ) -> bool {
        if let Some(home_base_pos) = brain_component.home_base {
            let territorial_threats: Vec<_> = hostile_players
                .iter()
                .filter(|&id| {
                    world
                        .get_component::<Position>(*id as usize)
                        .map_or(false, |p| {
                            p.x.abs_diff(home_base_pos.x) <= crate::config::DEFENSE_RADIUS
                                && p.y.abs_diff(home_base_pos.y) <= crate::config::DEFENSE_RADIUS
                        })
                })
                .copied()
                .collect();
            if !territorial_threats.is_empty() {
                if (health.current as f32 / health.max as f32)
                    < crate::config::CRITICAL_HEALTH_RATIO
                {
                    self.set_goal(brain_component, Goal::Flee);
                    return true;
                } else {
                    if let Some(id) =
                        self.find_closest_threat(world, player_pos, &territorial_threats)
                    {
                        self.set_goal(brain_component, Goal::Attack(id));
                        return true;
                    }
                }
            }
        }
        false
    }

    fn handle_standard_threats(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        player_pos: &Position,
        health: &Health,
        hostile_players: &[u32],
    ) -> bool {
        if hostile_players.len() > 1
            || (health.current as f32 / health.max as f32) < crate::config::STANDARD_HEALTH_RATIO
        {
            self.set_goal(brain_component, Goal::Flee);
            return true;
        } else {
            if let Some(id) = self.find_closest_threat(world, player_pos, hostile_players) {
                self.set_goal(brain_component, Goal::Attack(id));
                return true;
            }
        }
        false
    }

    fn find_closest_threat(
        &self,
        world: &World,
        player_pos: &Position,
        threats: &[u32],
    ) -> Option<u32> {
        threats
            .iter()
            .min_by_key(|&id| {
                world
                    .get_component::<Position>(*id as usize)
                    .map_or(u32::MAX, |p| {
                        p.x.abs_diff(player_pos.x) + p.y.abs_diff(player_pos.y)
                    })
            })
            .copied()
    }

    fn set_goal(&self, brain_component: &mut BrainComponent, goal: Goal) {
        brain_component.current_goal = Some(goal);
        brain_component.current_path = None;
    }

    fn execute_attack_goal(
        &self,
        _entity: Entity,
        target_id: u32,
        _current_episode: u32,
    ) -> Result<Option<BrainAction>, SimulationError> {
        Ok(Some(BrainAction::Attack(
            crate::components::WantsToAttack {
                target: target_id as usize,
            },
        )))
    }

    fn execute_flee_goal(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        entity: Entity,
        _current_episode: u32,
    ) -> Result<Option<BrainAction>, SimulationError> {
        let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) else {
            return Ok(None);
        };
        let hostile_positions: Vec<_> = brain_component
            .player_memories
            .iter()
            .filter(|(_, mem)| mem.relationship == RelationshipStatus::Hostile)
            .filter_map(|(id, _)| world.get_component::<Position>(*id as usize).cloned())
            .collect();
        if hostile_positions.is_empty() {
            brain_component.current_goal = None;
            return Ok(None);
        }
        let avg_threat_x = hostile_positions.iter().map(|p| p.x as f32).sum::<f32>()
            / hostile_positions.len() as f32;
        let avg_threat_y = hostile_positions.iter().map(|p| p.y as f32).sum::<f32>()
            / hostile_positions.len() as f32;
        let flee_vec_x = player_pos.x as f32 - avg_threat_x;
        let flee_vec_y = player_pos.y as f32 - avg_threat_y;
        let norm = (flee_vec_x.powi(2) + flee_vec_y.powi(2)).sqrt();
        let (flee_dx, flee_dy) = if norm > 0.0 {
            (
                (flee_vec_x / norm).round() as i32,
                (flee_vec_y / norm).round() as i32,
            )
        } else {
            (0, 0)
        };
        Ok(Some(BrainAction::Move(crate::components::Velocity {
            dx: flee_dx,
            dy: flee_dy,
        })))
    }

    fn execute_explore_goal(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        entity: Entity,
        _current_episode: u32,
    ) -> Result<Option<BrainAction>, SimulationError> {
        if brain_component.current_path.is_some() {
            return Ok(None);
        }
        let mut unvisited = Vec::new();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if brain_component.mental_map[y as usize][x as usize].is_none() {
                    unvisited.push((x, y));
                }
            }
        }
        if !unvisited.is_empty() {
            let target_idx = rand::thread_rng().gen_range(0..unvisited.len());
            let target_pos = unvisited[target_idx];
            if let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) {
                if let Some(path) = pathfinding::find_path(
                    (player_pos.x, player_pos.y),
                    target_pos,
                    &brain_component.mental_map,
                ) {
                    brain_component.current_path = Some(path);
                }
            }
        }
        Ok(None)
    }

    fn execute_stockpile_goal(
        &self,
        brain_component: &mut BrainComponent,
        world: &World,
        entity: Entity,
        resource: &str,
        _current_episode: u32,
    ) -> Result<Option<BrainAction>, SimulationError> {
        let Some(home_base_pos) = brain_component.home_base else {
            brain_component.current_goal = None;
            return Ok(None);
        };
        if let Some((chest_entity, chest_pos)) = self.find_closest_chest(world, &home_base_pos) {
            if let Some(player_pos) = world.get_component::<Position>(entity).map(|p| *p) {
                let (dx, dy) = (
                    (player_pos.x as i32 - chest_pos.x as i32).abs(),
                    (player_pos.y as i32 - chest_pos.y as i32).abs(),
                );
                if dx <= 1 && dy <= 1 {
                    return Ok(Some(BrainAction::Store(WantsToStoreItem {
                        item_name: resource.to_string(),
                        quantity: 1, // Simplified
                        target_chest: chest_entity,
                    })));
                } else if brain_component.current_path.is_none() {
                    if let Some(path) = pathfinding::find_path(
                        (player_pos.x, player_pos.y),
                        (chest_pos.x, chest_pos.y),
                        &brain_component.mental_map,
                    ) {
                        brain_component.current_path = Some(path);
                    }
                }
            }
        } else {
            brain_component.current_goal = None;
        }
        Ok(None)
    }

    fn find_closest_chest(&self, world: &World, pos: &Position) -> Option<(Entity, Position)> {
        (0..world.entities.len())
            .filter_map(|e| world.get_component::<Chest>(e).map(|_| e))
            .filter_map(|e| world.get_component::<Position>(e).map(|p| (e, *p)))
            .min_by_key(|(_, chest_pos)| chest_pos.x.abs_diff(pos.x) + chest_pos.y.abs_diff(pos.y))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{BrainComponent, Inventory, Position};
    use crate::config::{DISCOUNT_FACTOR, EPSILON, LEARNING_RATE};
    use crate::ecs::World;
    use crate::recipes::RecipeManager;
    use std::env;
    use std::sync::Arc;

    fn create_test_brain() -> Result<Brain, SimulationError> {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| SimulationError::UnwrapFailed(e.to_string()))?;
        let recipe_manager = Arc::new(RecipeManager::new(&format!(
            "{}/data/recipes.json",
            manifest_dir
        )));
        Ok(Brain::new(
            recipe_manager,
            LEARNING_RATE,
            DISCOUNT_FACTOR,
            EPSILON,
        ))
    }

    #[test]
    fn test_is_goal_complete() -> Result<(), SimulationError> {
        let brain = create_test_brain()?;
        let mut world = World::new();
        let player_entity = world.create_entity();
        let brain_component = BrainComponent::new();
        let mut inventory = Inventory::new();
        inventory.add_item("wood", 11);
        world.add_component(player_entity, inventory)?;

        let goal = Goal::GatherResource("wood".to_string());
        assert!(brain.is_goal_complete(&brain_component, &world, player_entity, &goal));
        Ok(())
    }

    #[test]
    fn test_planning_with_known_resource() -> Result<(), SimulationError> {
        let brain = create_test_brain()?;
        let mut world = World::new();
        let player_entity = world.create_entity();
        let mut brain_component = BrainComponent::new();
        brain_component
            .known_resources
            .entry("stone".to_string())
            .or_default()
            .insert(Position { x: 10, y: 10 });
        world.add_component(player_entity, Inventory::new())?;

        let goal = Goal::CraftItem("stone_axe".to_string());
        let plan = brain.plan_goal(&brain_component, &world, player_entity, &goal)?;

        // The recipe for stone_axe requires wood and stone. The AI knows where stone is,
        // but not wood. So the plan should include exploring for wood, gathering wood,
        // gathering stone, and finally crafting the axe.
        assert!(plan.contains(&Goal::CraftItem("stone_axe".to_string())));
        assert!(plan.contains(&Goal::GatherResource("stone".to_string())));
        assert!(plan.contains(&Goal::GatherResource("wood".to_string())));
        assert!(plan.contains(&Goal::Explore));
        Ok(())
    }
}
