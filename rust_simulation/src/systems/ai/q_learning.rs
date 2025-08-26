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
        });
}
