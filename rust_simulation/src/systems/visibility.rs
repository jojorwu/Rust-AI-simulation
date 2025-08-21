use crate::components::Position;
use crate::fov;
use crate::lib::{IsDay, Player};
use crate::map::{Map, TileState};
use bevy_ecs::prelude::*;

pub fn visibility_system(
    mut query: Query<(&mut Player, &Position)>,
    map: Res<Map>,
    is_day: Res<IsDay>,
) {
    for (mut player, pos) in query.iter_mut() {
        // Step 1: Set all currently visible tiles to explored.
        // This makes the FOV feel like a "light" rather than permanently revealing the map.
        for y in 0..player.mental_map.height {
            for x in 0..player.mental_map.width {
                if player.mental_map.grid[y as usize][x as usize] == TileState::Visible {
                    player.mental_map.grid[y as usize][x as usize] = TileState::Explored;
                }
            }
        }

        // Step 2: Calculate the new field of view.
        // The vision radius is larger during the day.
        let fov_radius = if is_day.0 { 8 } else { 4 };
        let visible_tiles = fov::field_of_view(pos, fov_radius, &map);

        // Step 3: Mark all tiles in the new FOV as visible.
        for visible_pos in visible_tiles.iter() {
            if visible_pos.x < player.mental_map.width && visible_pos.y < player.mental_map.height {
                player.mental_map.grid[visible_pos.y as usize][visible_pos.x as usize] =
                    TileState::Visible;
            }
        }
    }
}
