use crate::{
    components::{intents::IntendsToBuild, Inventory, Position},
    events::Event,
    map::Map,
    RecipeManagerResource,
};
use bevy_ecs::prelude::*;
use log::error;

pub fn building_system(
    mut commands: Commands,
    mut event_writer: EventWriter<Event>,
    mut query: Query<(Entity, &Position, &mut Inventory, &IntendsToBuild)>,
    map: Res<Map>,
    recipe_manager: Res<RecipeManagerResource>,
) {
    for (entity, position, mut inventory, intends_to_build) in query.iter_mut() {
        // Check 1: Tile is suitable
        if !map.is_walkable(position.x, position.y) {
            // Silently fail and remove intent if tile is not suitable
            commands.entity(entity).remove::<IntendsToBuild>();
            continue;
        }

        // Check 2: We have the required resources
        let recipe_manager = &recipe_manager.0;
        match recipe_manager.get_required_resources(&intends_to_build.0, 1) {
            Ok(required) => {
                if inventory.remove_resources(&required) {
                    // If all checks pass, send the event
                    event_writer.send(Event::BuildRequest {
                        builder: entity,
                        structure: intends_to_build.0.clone(),
                        position: *position,
                    });
                }
            }
            Err(e) => error!(
                "Could not get resources for building {}: {}",
                intends_to_build.0, e
            ),
        }

        // Always remove the intent to prevent getting stuck
        commands.entity(entity).remove::<IntendsToBuild>();
    }
}
