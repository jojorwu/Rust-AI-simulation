use crate::brain::{HighLevelState, Goal};
use crate::components::BrainComponent;
use crate::events::Event;
use crate::BrainResource;
use bevy_ecs::prelude::*;

pub fn update_q_table_system(
    mut brain_query: Query<&mut BrainComponent>,
    mut event_reader: EventReader<Event>,
    brain_res: Res<BrainResource>,
) {
    let brain = &brain_res.0;
    for event in event_reader.read() {
        if let Event::GoalCompleted {
            entity,
            prev_state,
            goal,
            new_state,
            reward,
        } = event
        {
            if let Ok(mut brain_component) = brain_query.get_mut(*entity) {
                let old_q_value = brain_component
                    .goal_q_table
                    .get(prev_state)
                    .and_then(|q| q.get(goal))
                    .cloned()
                    .unwrap_or(0.0);
                let max_future_q = brain_component
                    .goal_q_table
                    .get(new_state)
                    .map(|q| {
                        q.values()
                            .cloned()
                            .max_by(|a, b| a.total_cmp(b))
                            .unwrap_or(0.0)
                    })
                    .unwrap_or(0.0);
                let new_q_value = old_q_value
                    + brain.learning_rate * (reward + brain.discount_factor * max_future_q - old_q_value);
                brain_component
                    .goal_q_table
                    .entry(prev_state.clone())
                    .or_default()
                    .insert(goal.clone(), new_q_value);
            }
        }
    }
}
