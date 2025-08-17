use crate::map::Map;
use crate::components::Position;
use std::collections::HashSet;

/// Calculates the Field of View for a given position and radius using recursive shadowcasting.
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
    start: f32,
    end: f32,
    player_pos: &Position,
    radius: i32,
    octant: u8,
    map: &Map,
    visible_tiles: &mut HashSet<Position>,
) {
    if row > radius { return; }

    let mut new_start = 0.0;
    let mut prev_tile_was_wall = false;

    for col in 0..=row {
        let (dx, dy) = transform_octant(col, row, octant);
        let x = player_pos.x as i32 + dx;
        let y = player_pos.y as i32 + dy;

        if x < 0 || x >= map.width as i32 || y < 0 || y >= map.height as i32 {
            continue;
        }

        let pos = Position { x: x as u32, y: y as u32 };
        let in_radius = (dx*dx + dy*dy) <= radius*radius;

        let top_slope = if col == 0 { 1.0 } else { (col as f32 - 0.5) / (row as f32 - 0.5) };
        let bottom_slope = (col as f32 + 0.5) / (row as f32 + 0.5);

        let mut in_view = top_slope >= end && bottom_slope <= start;
        if !in_view && (top_slope > end || bottom_slope < start) {
            in_view = (top_slope <= start && top_slope >= end) || (bottom_slope <= start && bottom_slope >= end) || (top_slope >= start && bottom_slope <= end);
        }

        if in_view {
            if in_radius {
                visible_tiles.insert(pos);
            }

            let current_tile_is_wall = is_opaque(pos, map);
            if current_tile_is_wall {
                if !prev_tile_was_wall {
                    new_start = top_slope;
                }
                prev_tile_was_wall = true;
            } else {
                if prev_tile_was_wall {
                    scan(row + 1, new_start, top_slope, player_pos, radius, octant, map, visible_tiles);
                }
                prev_tile_was_wall = false;
            }
        }
    }

    if !prev_tile_was_wall {
        scan(row + 1, start, end, player_pos, radius, octant, map, visible_tiles);
    }
}


fn is_opaque(pos: Position, map: &Map) -> bool {
    if pos.x >= map.width || pos.y >= map.height {
        return true;
    }
    map.grid[pos.y as usize][pos.x as usize].tile_type == '#'
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
