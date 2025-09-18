use crate::components::Position;
use crate::map::Map;
use std::collections::HashSet;

/// Calculates the Field of View for a given position and radius using a standard recursive shadowcasting implementation.
pub fn field_of_view(player_pos: &Position, radius: i32, map: &Map) -> HashSet<Position> {
    let mut visible_tiles = HashSet::new();
    visible_tiles.insert(*player_pos);

    for octant in 0..8 {
        let mut context = ScanContext {
            player_pos,
            radius,
            octant,
            map,
            visible_tiles: &mut visible_tiles,
        };
        scan(1, 1.0, 0.0, &mut context);
    }

    visible_tiles
}

/// A context struct to hold the data that doesn't change during the scan recursion.
struct ScanContext<'a> {
    player_pos: &'a Position,
    radius: i32,
    octant: u8,
    map: &'a Map,
    visible_tiles: &'a mut HashSet<Position>,
}

fn scan(row: i32, start_slope: f32, end_slope: f32, context: &mut ScanContext) {
    if start_slope < end_slope {
        return;
    }

    let mut next_start_slope = start_slope;
    let mut last_col = -1;

    for col in 0..=row {
        if last_col != -1 && col < last_col {
            continue;
        }

        let (dx, dy) = transform_octant(col, row, context.octant);
        let x = context.player_pos.x as i32 + dx;
        let y = context.player_pos.y as i32 + dy;

        if x < 0 || x >= context.map.width as i32 || y < 0 || y >= context.map.height as i32 {
            continue;
        }

        let pos = Position {
            x: x as u32,
            y: y as u32,
        };
        if (dx * dx + dy * dy) > context.radius * context.radius {
            continue;
        }

        let slope = (2.0 * col as f32 - 1.0) / (2.0 * row as f32);
        let next_slope = (2.0 * col as f32 + 1.0) / (2.0 * row as f32);

        if slope > start_slope || next_slope < end_slope {
            continue;
        }

        context.visible_tiles.insert(pos);

        if is_opaque(pos, context.map) {
            if col > 0 {
                scan(row + 1, next_start_slope, slope, context);
            }
            next_start_slope = next_slope;
        }
    }
}

fn is_opaque(pos: Position, map: &Map) -> bool {
    if let Some(tile) = map.get_tile(pos.x, pos.y) {
        matches!(tile.tile_type, '#' | 'f' | 'T')
    } else {
        true
    }
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
