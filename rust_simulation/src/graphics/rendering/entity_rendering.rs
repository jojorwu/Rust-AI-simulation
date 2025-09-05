use crate::{
    animals::pig::Pig,
    components::Position,
    player::Player,
};
use bevy::prelude::*;

pub const TILE_SIZE: f32 = 32.0;

#[derive(Component)]
pub struct RenderedEntity(pub Entity);

pub struct EntityRenderingPlugin;

impl Plugin for EntityRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_entity_sprites)
            .add_systems(Update, update_entity_positions);
    }
}

pub fn setup_entity_sprites(
    mut commands: Commands,
    player_query: Query<(Entity, &Position), With<Player>>,
    pig_query: Query<(Entity, &Position), With<Pig>>,
) {
    // Spawn player sprite
    for (player_entity, position) in player_query.iter() {
        let spawned_entity = commands
            .spawn((SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    position.x as f32 * TILE_SIZE,
                    position.y as f32 * TILE_SIZE,
                    1.0,
                ),
                ..default()
            },))
            .id();
        commands
            .entity(spawned_entity)
            .insert(RenderedEntity(player_entity));
    }

    // Spawn pig sprites
    for (pig_entity, position) in pig_query.iter() {
        let spawned_entity = commands
            .spawn((SpriteBundle {
                sprite: Sprite {
                    color: Color::PINK,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    position.x as f32 * TILE_SIZE,
                    position.y as f32 * TILE_SIZE,
                    1.0,
                ),
                ..default()
            },))
            .id();
        commands
            .entity(spawned_entity)
            .insert(RenderedEntity(pig_entity));
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
