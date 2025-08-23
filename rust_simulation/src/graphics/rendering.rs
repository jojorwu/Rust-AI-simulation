use bevy::prelude::*;
use crate::map::{Map, Tile};
use crate::player::Player;
use crate::components::Position;
use crate::Game;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_map_and_entities)
            .add_systems(Update, update_entity_positions);
    }
}

pub(super) const TILE_SIZE: f32 = 32.0;

#[derive(Component)]
pub(super) struct RenderedEntity(pub(super) Entity);

pub(super) fn setup_map_and_entities(mut commands: Commands, mut game: ResMut<Game>) {
    let map = game.world.get_resource::<Map>().unwrap();

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
    let mut player_query = game.world.query_filtered::<Entity, With<Player>>();
    for entity in player_query.iter(&game.world) {
        let position = game.world.get::<Position>(entity).unwrap();
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
        commands.entity(spawned_entity).insert(RenderedEntity(entity));
    }
}

pub(super) fn update_entity_positions(game: Res<Game>, mut query: Query<(&mut Transform, &RenderedEntity)>) {
    for (mut transform, rendered_entity) in query.iter_mut() {
        if let Some(position) = game.world.get::<Position>(rendered_entity.0) {
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
