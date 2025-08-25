use crate::components::{ai::GoalQTable, BrainComponent};
use crate::events::Event;
use bevy_ecs::prelude::*;
use dashmap::DashMap;
use rayon::prelude::*;

pub fn update_q_table_system(
    mut q_table_query: Query<(Entity, &BrainComponent, &mut GoalQTable)>,
    mut event_reader: EventReader<Event>,
) {
    let events: Vec<_> = event_reader.read().collect();
    let events_by_entity: DashMap<Entity, Vec<&Event>> = DashMap::new();

    events.par_iter().for_each(|event| {
        if let Event::GoalCompleted { entity, .. } = event {
            events_by_entity.entry(*entity).or_default().push(event);
        }
    });

    q_table_query
        .par_iter_mut()
        .for_each(|(entity, brain, mut q_table)| {
            if let Some(events) = events_by_entity.get(&entity) {
                for event in events.iter() {
                    if let Event::GoalCompleted {
                        prev_state,
                        goal,
                        new_state,
                        reward,
                        ..
                    } = event
                    {
                        // Find the Q-values for the previous state.
                        let old_q_value = q_table
                            .0
                            .iter()
                            .find(|(s, _)| s == prev_state)
                            .and_then(|(_, q)| q.get(goal))
                            .copied()
                            .unwrap_or(0.0);

                        // Find the max Q-value for the new state.
                        let max_future_q = q_table
                            .0
                            .iter()
                            .find(|(s, _)| s == new_state)
                            .map(|(_, q)| {
                                q.values()
                                    .max_by(|a, b| a.total_cmp(b))
                                    .copied()
                                    .unwrap_or(0.0)
                            })
                            .unwrap_or(0.0);

                        let new_q_value = old_q_value
                            + brain.learning_rate
                                * (reward + brain.discount_factor * max_future_q - old_q_value);

                        // Update or insert the Q-value for the previous state and goal.
                        if let Some((_, q_map)) = q_table.0.iter_mut().find(|(s, _)| s == prev_state) {
                            // If the state already exists, update the goal's Q-value.
                            q_map.insert(goal.clone(), new_q_value);
                        } else {
                            // If the state doesn't exist, add it with the new goal and Q-value.
                            let mut new_q_map = std::collections::HashMap::new();
                            new_q_map.insert(goal.clone(), new_q_value);
                            q_table.0.push((prev_state.clone(), new_q_map));
                        }
                    }
                }
            }
        });
}
