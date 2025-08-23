use crate::map::{Map, MapChunk, CHUNK_SIZE};
use crate::player::Player;
use crate::components::Position;
use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_chunk_meshes)
            .add_systems(Update, update_entity_positions);
    }
}

pub const TILE_SIZE: f32 = 32.0;

#[derive(Component)]
pub struct RenderedEntity(pub Entity);

use bevy::render::render_asset::RenderAssetUsages;

fn create_chunk_mesh(chunk: &MapChunk, chunk_pos: (u32, u32)) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut vertices = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    let mut index_offset = 0;

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = &chunk.tiles[y as usize][x as usize];
            let color = tile_type_to_color(tile);
            let x_pos = (chunk_pos.0 * CHUNK_SIZE + x) as f32 * TILE_SIZE;
            let y_pos = (chunk_pos.1 * CHUNK_SIZE + y) as f32 * TILE_SIZE;

            vertices.push([x_pos, y_pos, 0.0]);
            vertices.push([x_pos + TILE_SIZE, y_pos, 0.0]);
            vertices.push([x_pos, y_pos + TILE_SIZE, 0.0]);
            vertices.push([x_pos + TILE_SIZE, y_pos + TILE_SIZE, 0.0]);

            for _ in 0..4 {
                colors.push([color.r(), color.g(), color.b(), color.a()]);
            }

            indices.push(index_offset);
            indices.push(index_offset + 1);
            indices.push(index_offset + 2);
            indices.push(index_offset + 1);
            indices.push(index_offset + 3);
            indices.push(index_offset + 2);

            index_offset += 4;
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(mesh::Indices::U32(indices));
    mesh
}

pub fn setup_chunk_meshes(
    mut commands: Commands,
    map: Res<Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<(Entity, &Position), With<Player>>,
) {
    for (chunk_y, chunk_row) in map.chunks.iter().enumerate() {
        for (chunk_x, chunk) in chunk_row.iter().enumerate() {
            let chunk_pos = (chunk_x as u32, chunk_y as u32);
            let chunk_mesh = create_chunk_mesh(&chunk.lock().unwrap(), chunk_pos);
            commands.spawn(ColorMesh2dBundle {
                mesh: meshes.add(chunk_mesh).into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                ..default()
            });
        }
    }

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

fn tile_type_to_color(tile: &crate::map::Tile) -> Color {
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
