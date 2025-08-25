use crate::components::{BrainComponent, WantsToCraft, intents::IntendsToCraft};
use bevy_ecs::prelude::*;

pub fn craft_action_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BrainComponent, &IntendsToCraft)>,
) {
    for (entity, mut brain_component, intent) in query.iter_mut() {
        let item_name = &intent.0;
        commands.entity(entity).insert(WantsToCraft {
            item_name: item_name.to_string(),
        });

        // Crafting is a single-tick action, so the goal is complete.
        brain_component.current_goal = None;
        commands.entity(entity).remove::<IntendsToCraft>();
    }
}
