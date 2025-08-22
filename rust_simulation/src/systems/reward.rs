use crate::{
    brain::Goal,
    components::{ai::PlayerMemories, BrainComponent, Equipped, Health, Inventory},
    config,
    events::Event,
    systems::ai::goal_selection::get_high_level_state,
    IsDay,
};
use bevy_ecs::prelude::*;

pub fn reward_system(
    mut event_reader: EventReader<Event>,
    mut event_writer: EventWriter<Event>,
    query: Query<(
        &BrainComponent,
        &Health,
        &Inventory,
        &Equipped,
        &PlayerMemories,
    )>,
    is_day: Res<IsDay>,
) {
    for event in event_reader.read() {
        let (entity, goal, reward) = match event {
            Event::ItemCrafted { entity, item_name } => {
                let reward = match item_name.as_str() {
                    "stone_axe" => config::CRAFT_STONE_AXE_REWARD,
                    _ => config::CRAFT_REWARD,
                };
                (*entity, Goal::CraftItem(item_name.clone()), reward)
            }
            Event::ResourceGathered { entity, resource, .. } => {
                let reward = match resource.as_str() {
                    "iron_ore" => config::GATHER_IRON_ORE_REWARD,
                    _ => config::GATHER_REWARD,
                };
                (*entity, Goal::GatherResource(resource.clone()), reward)
            }
            Event::ToolEquipped { entity, tool_name } => (
                *entity,
                Goal::Equip(tool_name.clone()),
                config::EQUIP_TOOL_REWARD,
            ),
            _ => continue, // Ignore other events
        };

        if let Ok((_brain, health, inventory, equipped, memories)) = query.get(entity) {
            // A more advanced system would store the state when the goal is set to calculate a more accurate Q-value.
            // For now, we just use the current state as both previous and new.
            let current_state = get_high_level_state(health, inventory, memories, equipped, is_day.0);

            event_writer.send(Event::GoalCompleted {
                entity,
                prev_state: current_state.clone(),
                goal,
                new_state: current_state,
                reward,
            });
        }
    }
}
