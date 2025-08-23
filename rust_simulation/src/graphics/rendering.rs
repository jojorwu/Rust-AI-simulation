use bevy::prelude::*;
use crate::map::{Map, Tile};
use crate::player::Player;
use crate::components::Position;
use crate::SimulationSet;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_map_and_entities.after(SimulationSet::Setup))
            .add_systems(Update, update_entity_positions);
    }
}

pub const TILE_SIZE: f32 = 32.0;

#[derive(Component)]
pub struct RenderedEntity(pub Entity);

fn setup_map_and_entities(
    mut commands: Commands,
    map: Res<Map>,
    player_query: Query<(Entity, &Position), With<Player>>,
) {
    // Spawn map tiles
    for y in 0..map.height {
        for x in 0..map.width {
            if let Some(tile) = map.get_tile(x, y) {
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: tile_type_to_color(&tile),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0),
                    ..default()
                });
            }
        }
    }

    // Spawn player sprite
    for (player_entity, position) in player_query.iter() {
        let spawned_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x as f32 * TILE_SIZE, position.y as f32 * TILE_SIZE, 1.0),
                ..default()
            },
        )).id();
        commands.entity(spawned_entity).insert(RenderedEntity(player_entity));
    }
}

fn update_entity_positions(
    mut sprite_query: Query<(&mut Transform, &RenderedEntity)>,
    position_query: Query<&Position>,
) {
    for (mut transform, rendered_entity) in sprite_query.iter_mut() {
        if let Ok(position) = position_query.get(rendered_entity.0) {
            transform.translation.x = position.x as f32 * TILE_SIZE;
            transform.translation.y = position.y as f32 * TILE_SIZE;
        }
    }
}

fn tile_type_to_color(tile: &Tile) -> Color {
    match tile.tile_type {
        '.' => Color::rgb(0.2, 0.8, 0.2), // Green for grass
        'f' => Color::rgb(0.1, 0.5, 0.1), // Dark green for forest
        'M' => Color::rgb(0.5, 0.5, 0.5), // Grey for mountain
        'T' => Color::rgb(0.0, 0.4, 0.0), // Darker green for trees
        '~' => Color::rgb(0.3, 0.3, 0.9), // Blue for water
        '#' => Color::rgb(0.6, 0.4, 0.2), // Brown for road
        'O' => Color::rgb(0.7, 0.7, 0.7), // Light grey for rock
        _ => Color::BLACK,
    }
}
