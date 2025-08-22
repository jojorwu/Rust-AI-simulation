use crate::components::{ai::GoalQTable, BrainComponent};
use crate::events::Event;
use bevy_ecs::prelude::*;

pub fn update_q_table_system(
    mut q_table_query: Query<(&BrainComponent, &mut GoalQTable)>,
    mut event_reader: EventReader<Event>,
) {
    for event in event_reader.read() {
        if let Event::GoalCompleted {
            entity,
            prev_state,
            goal,
            new_state,
            reward,
        } = event
        {
            if let Ok((brain, mut q_table)) = q_table_query.get_mut(*entity) {
                let old_q_value = q_table
                    .0
                    .get(prev_state)
                    .and_then(|q| q.get(goal))
                    .cloned()
                    .unwrap_or(0.0);
                let max_future_q = q_table
                    .0
                    .get(new_state)
                    .map(|q| {
                        q.values()
                            .cloned()
                            .max_by(|a, b| a.total_cmp(b))
                            .unwrap_or(0.0)
                    })
                    .unwrap_or(0.0);
                let new_q_value = old_q_value
                    + brain.learning_rate
                        * (reward + brain.discount_factor * max_future_q - old_q_value);
                q_table
                    .0
                    .entry(prev_state.clone())
                    .or_default()
                    .insert(goal.clone(), new_q_value);
            }
        }
    }
}
