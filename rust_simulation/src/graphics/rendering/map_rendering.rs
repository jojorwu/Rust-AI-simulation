use crate::map::{Map, MapChunk, Tile, CHUNK_SIZE};
use bevy::{
    prelude::*,
    render::{mesh, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use rayon::prelude::*;
use std::collections::HashSet;
use crate::player::Player;
use crate::components::Position;

use super::TILE_SIZE;

const VIEW_DISTANCE: u32 = 2;

#[derive(Resource, Default)]
pub struct VisibleChunks(pub HashSet<(u32, u32)>);

#[derive(Component)]
struct LoadedChunk {
    x: u32,
    y: u32,
}

pub struct MapRenderingPlugin;

impl Plugin for MapRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisibleChunks>()
            .add_systems(Update, (chunk_visibility_system, load_new_chunks_system, unload_old_chunks_system));
    }
}

fn chunk_visibility_system(
    player_query: Query<&Position, With<Player>>,
    mut visible_chunks: ResMut<VisibleChunks>,
    map: Res<Map>,
) {
    if let Ok(player_pos) = player_query.get_single() {
        let player_chunk_x = player_pos.x / CHUNK_SIZE;
        let player_chunk_y = player_pos.y / CHUNK_SIZE;

        let mut new_visible_chunks = HashSet::new();
        for y in (player_chunk_y.saturating_sub(VIEW_DISTANCE))..=(player_chunk_y + VIEW_DISTANCE) {
            for x in (player_chunk_x.saturating_sub(VIEW_DISTANCE))..=(player_chunk_x + VIEW_DISTANCE) {
                if x < map.width_in_chunks() && y < map.height_in_chunks() {
                    new_visible_chunks.insert((x, y));
                }
            }
        }

        if visible_chunks.0 != new_visible_chunks {
            visible_chunks.0 = new_visible_chunks;
        }
    }
}

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

fn load_new_chunks_system(
    mut commands: Commands,
    map: Res<Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    visible_chunks: Res<VisibleChunks>,
    loaded_chunks_query: Query<&LoadedChunk>,
) {
    let loaded_chunks: HashSet<(u32, u32)> = loaded_chunks_query
        .iter()
        .map(|chunk| (chunk.x, chunk.y))
        .collect();

    let chunks_to_load = visible_chunks
        .0
        .difference(&loaded_chunks)
        .cloned()
        .collect::<Vec<_>>();

    let chunks_with_data_to_load = chunks_to_load
        .into_iter()
        .map(|(x, y)| {
            let chunk_data = map.chunks[y as usize][x as usize].clone();
            (chunk_data, (x, y))
        })
        .collect::<Vec<_>>();

    let new_meshes: Vec<(Mesh, (u32, u32))> = chunks_with_data_to_load
        .into_par_iter()
        .map(|(chunk_data, pos)| {
            let chunk = chunk_data.lock().unwrap();
            (create_chunk_mesh(&chunk, pos), pos)
        })
        .collect();

    for (mesh, (x, y)) in new_meshes {
        commands.spawn((
            ColorMesh2dBundle {
                mesh: meshes.add(mesh).into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                ..default()
            },
            LoadedChunk { x, y },
        ));
    }
}

fn unload_old_chunks_system(
    mut commands: Commands,
    visible_chunks: Res<VisibleChunks>,
    loaded_chunks_query: Query<(Entity, &LoadedChunk)>,
) {
    for (entity, loaded_chunk) in loaded_chunks_query.iter() {
        if !visible_chunks.0.contains(&(loaded_chunk.x, loaded_chunk.y)) {
            commands.entity(entity).despawn();
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
