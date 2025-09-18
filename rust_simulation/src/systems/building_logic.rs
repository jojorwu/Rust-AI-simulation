use crate::{
    components::{
        intents::{CheckResources, HasResources, TileIsSuitable},
        Inventory, Position,
    },
    events::Event,
    map::{CHUNK_SIZE, Map},
    RecipeManagerResource,
};
use bevy_ecs::prelude::*;

use log::error;

pub fn check_resources_system(
    mut commands: Commands,
    query: Query<(Entity, &Inventory, &CheckResources)>,
    recipe_manager: Res<RecipeManagerResource>,
) {
    let recipe_manager = &recipe_manager.0;
    for (entity, inventory, check_resources) in query.iter() {
        match recipe_manager.get_required_resources(&check_resources.0, 1) {
            Ok(required) => {
                if inventory.has_resources(&required) {
                    commands.entity(entity).insert(HasResources);
                }
            }
            Err(e) => error!(
                "Could not get resources for {}: {}",
                check_resources.0, e
            ),
        }
    }
}

pub fn check_tile_system(
    mut commands: Commands,
    query: Query<(Entity, &Position), With<HasResources>>,
    map: Res<Map>,
) {
    for (entity, position) in query.iter() {
        if let Some((chunk_x, chunk_y)) = map.get_chunk_index(position.x, position.y) {
            if let Ok(chunk) = map.chunks[chunk_y][chunk_x].try_lock() {
                let local_x = (position.x % CHUNK_SIZE) as usize;
                let local_y = (position.y % CHUNK_SIZE) as usize;
                let tile = &chunk.tiles[local_y][local_x];
                if tile.tile_type == '.' {
                    commands.entity(entity).insert(TileIsSuitable);
                }
            }
        }
    }
}

pub fn build_system(
    mut commands: Commands,
    mut event_writer: EventWriter<Event>,
    mut query: Query<(Entity, &Position, &CheckResources, &mut Inventory), With<TileIsSuitable>>,
    recipe_manager: Res<RecipeManagerResource>,
) {
    let recipe_manager = &recipe_manager.0;
    for (entity, position, check_resources, mut inventory) in query.iter_mut() {
        if let Ok(required) = recipe_manager.get_required_resources(&check_resources.0, 1) {
            if inventory.remove_resources(&required) {
                event_writer.send(Event::BuildRequest {
                    builder: entity,
                    structure: check_resources.0.clone(),
                    position: *position,
                });
                commands
                    .entity(entity)
                    .remove::<(CheckResources, HasResources, TileIsSuitable)>();
            }
        }
    }
}
