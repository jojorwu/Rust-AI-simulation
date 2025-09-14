use crate::{
    components::ai::KnownResources,
    events::Event,
};
use bevy_ecs::prelude::*;

/// This system listens for `ResourceDepleted` events and updates the `KnownResources`
/// of all agents to remove the depleted resource from their knowledge base.
pub fn update_known_resources_system(
    mut events: EventReader<Event>,
    mut query: Query<&mut KnownResources>,
) {
    for event in events.read() {
        if let Event::ResourceDepleted { resource, position } = event {
            for mut known_resources in query.iter_mut() {
                if let Some(positions) = known_resources.0.get_mut(resource) {
                    positions.retain(|&p| p != *position);
                }
            }
        }
    }
}
