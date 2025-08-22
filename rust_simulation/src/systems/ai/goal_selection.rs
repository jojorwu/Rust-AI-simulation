use crate::brain::Goal;
use crate::components::{intents::*, BrainComponent, Health, Inventory};
use crate::IsDay;
use bevy_ecs::prelude::*;
use log::info;

pub fn goal_selection_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &Health, &Inventory)>,
    is_day: Res<IsDay>,
) {
    for (entity, mut brain_component, health, inventory) in query.iter_mut() {
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
                    if let Some(goal) = &brain_component.current_goal {
                        info!("Entity {:?} selected new goal: {:?}", entity, goal);

                        // Add the corresponding intent component
                        match goal {
                            Goal::GatherResource(res) => {
                                commands.entity(entity).insert(IntendsToGather(res.clone()));
                            }
                            Goal::CraftItem(item) => {
                                commands.entity(entity).insert(IntendsToCraft(item.clone()));
                            }
                            Goal::Build(structure) => {
                                commands.entity(entity).insert(IntendsToBuild(structure.clone()));
                            }
                            Goal::Attack(target) => {
                                commands.entity(entity).insert(IntendsToAttack(*target));
                            }
                            Goal::Flee => {
                                commands.entity(entity).insert(IntendsToFlee);
                            }
                            Goal::Explore => {
                                commands.entity(entity).insert(IntendsToExplore);
                            }
                            Goal::Stockpile(res) => {
                                commands.entity(entity).insert(IntendsToStockpile(res.clone()));
                            }
                        }

                        brain_component.goal_commitment_ticks = crate::config::GOAL_COMMITMENT_TICKS;
                    }
                }
            }
        }
    }
}
