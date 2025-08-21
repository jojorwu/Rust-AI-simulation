use crate::components::{BrainComponent, Health, Inventory};
use crate::IsDay;
use bevy_ecs::prelude::*;

pub fn goal_selection_system(
    mut query: Query<(&mut BrainComponent, &Health, &Inventory)>,
    is_day: Res<IsDay>,
) {
    for (mut brain_component, health, inventory) in query.iter_mut() {
        if brain_component.current_goal.is_none() && brain_component.goal_commitment_ticks == 0 {
            let high_level_state =
                brain_component.get_high_level_state(health, inventory, is_day.0);
            if let Ok(new_high_level_goal) =
                brain_component.choose_goal(&high_level_state)
            {
                if let Ok(mut plan) =
                    brain_component.plan_goal(inventory, &new_high_level_goal)
                {
                    plan.reverse();
                    brain_component.goal_stack = plan;
                    brain_component.current_goal = brain_component.goal_stack.pop();
                    if brain_component.current_goal.is_some() {
                        brain_component.goal_commitment_ticks = crate::config::GOAL_COMMITMENT_TICKS;
                    }
                }
            }
        }
    }
}
