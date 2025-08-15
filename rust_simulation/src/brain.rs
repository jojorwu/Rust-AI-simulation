use std::collections::HashMap;
use rand::Rng;
use super::state::StateKey;
use super::errors::SimulationError;

pub struct Brain {
    pub actions: Vec<String>,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub epsilon: f64,
    pub q_table: HashMap<String, HashMap<String, f64>>,
}

impl Brain {
    pub fn new(actions: Vec<String>, learning_rate: f64, discount_factor: f64, epsilon: f64) -> Self {
        Brain {
            actions,
            learning_rate,
            discount_factor,
            epsilon,
            q_table: HashMap::new(),
        }
    }

    pub fn choose_action(&self, state: &StateKey) -> Result<String, SimulationError> {
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

    pub fn update_q_table(&mut self, state: &StateKey, action: &str, reward: f64, next_state: &StateKey) -> Result<(), SimulationError> {
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
            .insert(action.to_string(), new_value);

        Ok(())
    }
}
