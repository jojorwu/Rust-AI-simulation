use crate::{
    brain::HighLevelState,
    components::{
        ai::PlayerMemories, status::{Health, Hunger}, BrainComponent, Inventory
    },
    events::Event,
    IsDay,
};
use bevy_ecs::prelude::*;

pub fn goal_completion_system(
    mut query: Query<(Entity, &mut BrainComponent, &Health, &Hunger, &Inventory, &PlayerMemories)>,
    mut event_writer: EventWriter<Event>,
    is_day: Res<IsDay>,
) {
    for (entity, mut brain, health, hunger, inventory, player_memories) in query.iter_mut() {
        if brain.goal_stack.is_empty() && brain.current_goal.is_some() {
            if let (Some(prev_state), Some(goal)) = (brain.prev_state.clone(), brain.current_goal.clone()) {
                let new_state =
                    super::goal_selection::get_high_level_state(health, hunger, inventory, player_memories, is_day.0);

                // Simple reward system: 1.0 for any completed goal.
                let reward = 1.0;

                event_writer.send(Event::GoalCompleted {
                    entity,
                    prev_state,
                    goal,
                    new_state,
                    reward,
                });
            }

            brain.current_goal = None;
            brain.prev_state = None;
            brain.prev_goal = None;
        }
    }
}
