use crate::components::BrainComponent;
use crate::events::Event;
use bevy_ecs::prelude::*;

pub fn brain_event_handler_system(
    mut query: Query<(Entity, &mut BrainComponent)>,
    mut event_reader: EventReader<Event>,
) {
    for event in event_reader.read() {
        if let Event::FoundationBuilt { builder, position } = event {
            if let Ok((_, mut brain_component)) = query.get_mut(*builder) {
                if brain_component.home_base.is_none() {
                    brain_component.home_base = Some(*position);
                }
            }
        }
    }
}
