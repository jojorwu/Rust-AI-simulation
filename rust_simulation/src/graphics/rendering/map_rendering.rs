use crate::map::{Map, MapChunk, Tile, CHUNK_SIZE};
use bevy::{
    prelude::*,
    render::{mesh, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};

use super::TILE_SIZE;

pub struct MapRenderingPlugin;

impl Plugin for MapRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_map_meshes);
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

pub fn setup_map_meshes(
    mut commands: Commands,
    map: Res<Map>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
