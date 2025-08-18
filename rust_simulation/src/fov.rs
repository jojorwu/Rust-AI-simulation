use crate::map::Map;
use crate::components::Position;
use std::collections::HashSet;

/// Calculates the Field of View for a given position and radius using a standard recursive shadowcasting implementation.
pub fn field_of_view(
    player_pos: &Position,
    radius: i32,
    map: &Map,
) -> HashSet<Position> {
    let mut visible_tiles = HashSet::new();
    visible_tiles.insert(*player_pos);

    for octant in 0..8 {
        scan(1, 1.0, 0.0, player_pos, radius, octant, map, &mut visible_tiles);
    }

    visible_tiles
}

fn scan(
    row: i32,
    mut start_slope: f32,
    end_slope: f32,
    player_pos: &Position,
    radius: i32,
    octant: u8,
    map: &Map,
    visible_tiles: &mut HashSet<Position>,
) {
    if row > radius { return; }

    let mut prev_tile_was_wall = false;
    let last_col = -1;

    for col in 0..=row {
        if last_col != -1 && col < last_col { continue; }

        let (dx, dy) = transform_octant(col, row, octant);
        let x = player_pos.x as i32 + dx;
        let y = player_pos.y as i32 + dy;

        if x < 0 || x >= map.width as i32 || y < 0 || y >= map.height as i32 {
            continue;
        }

        let pos = Position { x: x as u32, y: y as u32 };
        let in_radius = (dx*dx + dy*dy) <= radius*radius;

        let top_slope = if col == 0 { 1.0 } else { (2.0 * col as f32 - 1.0) / (2.0 * row as f32 - 1.0) };
        let bottom_slope = (2.0 * col as f32 + 1.0) / (2.0 * row as f32 + 1.0);

        if start_slope < bottom_slope { continue; }
        if end_slope > top_slope { break; }

        if in_radius {
            visible_tiles.insert(pos);
        }

        let current_tile_is_wall = is_opaque(pos, map);
        if current_tile_is_wall {
            if !prev_tile_was_wall {
                if col > 0 {
                    scan(row + 1, start_slope, top_slope, player_pos, radius, octant, map, visible_tiles);
                }
            }
            prev_tile_was_wall = true;
            start_slope = bottom_slope;
        } else {
            prev_tile_was_wall = false;
        }
    }

    if !prev_tile_was_wall {
        scan(row + 1, start_slope, end_slope, player_pos, radius, octant, map, visible_tiles);
    }
}


fn is_opaque(pos: Position, map: &Map) -> bool {
    if pos.x >= map.width || pos.y >= map.height {
        return true;
    }
    let tile_type = map.grid[pos.y as usize][pos.x as usize].tile_type;
    tile_type == '#' || tile_type == 'f' || tile_type == 'T'
}

fn transform_octant(x: i32, y: i32, octant: u8) -> (i32, i32) {
    match octant {
        0 => (x, -y),
        1 => (y, -x),
        2 => (y, x),
        3 => (x, y),
        4 => (-x, y),
        5 => (-y, x),
        6 => (-y, -x),
        _ => (-x, -y),
    }
}
