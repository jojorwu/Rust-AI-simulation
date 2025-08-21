use crate::brain::Brain;
use crate::components::{BrainComponent, Health, Inventory};
use crate::{BrainResource, IsDay};
use bevy_ecs::prelude::*;

pub fn goal_selection_system(
    mut query: Query<(&mut BrainComponent, &Health, &Inventory)>,
    brain_res: Res<BrainResource>,
    is_day: Res<IsDay>,
) {
    let brain = &brain_res.0;
    for (mut brain_component, health, inventory) in query.iter_mut() {
        if brain_component.current_goal.is_none() && brain_component.goal_commitment_ticks == 0 {
            let high_level_state =
                brain.get_high_level_state(health, inventory, &brain_component, is_day.0);
            if let Ok(new_high_level_goal) =
                brain.choose_goal(&brain_component, &high_level_state)
            {
                if let Ok(mut plan) =
                    brain.plan_goal(&brain_component, inventory, &new_high_level_goal)
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
