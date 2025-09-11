use crate::components::{intents::IntendsToBuild, intents::CheckResources};
use bevy_ecs::prelude::*;

pub fn building_system(
    mut commands: Commands,
    query: Query<(Entity, &IntendsToBuild)>,
) {
    for (entity, intends_to_build) in query.iter() {
        commands.entity(entity).insert(CheckResources(intends_to_build.0.clone()));
        commands.entity(entity).remove::<IntendsToBuild>();
    }
}
