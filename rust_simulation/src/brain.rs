use std::collections::HashMap;
use rand::Rng;
use super::state::StateKey;
use super::errors::SimulationError;
use super::config::{WIDTH, HEIGHT};
use super::actions::Action; // Import Action

pub struct Brain {
    pub actions: Vec<Action>, // Use Action enum
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub epsilon: f64,
    pub q_table: HashMap<String, HashMap<Action, f64>>, // Use Action as key
    pub mental_map: Vec<Vec<Option<char>>>,
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
        }
    }

    pub fn choose_action(&self, state: &StateKey) -> Result<Action, SimulationError> {
        if rand::thread_rng().r#gen::<f64>() < self.epsilon {
            // Explore
            let index = rand::thread_rng().gen_range(0..self.actions.len());
            Ok(self.actions[index].clone())
        } else {
            // Exploit
            let state_key_str = serde_json::to_string(state)?;
            if let Some(q_values) = self.q_table.get(&state_key_str) {
                // Find the action with the highest Q-value
                let best_action = q_values
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(action, _)| action.clone())
                    .unwrap_or_else(|| self.actions[0].clone());
                Ok(best_action)
            } else {
                // If state is unknown, choose randomly
                let index = rand::thread_rng().gen_range(0..self.actions.len());
                Ok(self.actions[index].clone())
            }
        }
    }

    pub fn update_q_table(&mut self, state: &StateKey, action: &Action, reward: f64, next_state: &StateKey) -> Result<(), SimulationError> {
        let state_key_str = serde_json::to_string(state)?;
        let next_state_key_str = serde_json::to_string(next_state)?;

        let old_value = self.q_table
            .get(&state_key_str)
            .and_then(|actions| actions.get(action))
            .cloned()
            .unwrap_or(0.0);

        let next_max = self.q_table
            .get(&next_state_key_str)
            .map_or(0.0, |actions| {
                actions.values().cloned().fold(f64::NEG_INFINITY, f64::max)
            });

        let new_value = old_value + self.learning_rate * (reward + self.discount_factor * next_max - old_value);

        self.q_table
            .entry(state_key_str)
            .or_insert_with(HashMap::new)
            .insert(action.clone(), new_value); // action needs to be cloned

        Ok(())
    }
}
