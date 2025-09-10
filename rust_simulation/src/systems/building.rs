use crate::components::{intents::IntendsToBuild, intents::CheckResources};
use bevy_ecs::prelude::*;

pub fn building_system(
    mut commands: Commands,
    query: Query<(Entity, &IntendsToBuild)>,
) {
    for (entity, intends_to_build) in query.iter() {
        commands.entity(entity).insert(CheckResources(intends_to_build.structure.clone()));
        // The IntendsToBuild component will be removed later by the main build_system
        // after the entire check process is complete.
    }
}
