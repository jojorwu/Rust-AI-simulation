use crate::brain::MemoryTile;
use crate::components::{
    ai::{ExplorationFrontier, MentalMap},
    Position,
};
use crate::fov;
use crate::map::Map;
use bevy_ecs::prelude::*;
use std::sync::Arc;

const VISION_RADIUS: i32 = 8;

pub fn visibility_system(
    map: Res<Map>,
    mut query: Query<(&Position, &mut MentalMap, &mut ExplorationFrontier)>,
) {
    for (pos, mut mental_map, mut exploration_frontier) in query.iter_mut() {
        let visible_tiles = fov::field_of_view(pos, VISION_RADIUS, &map);
        let mut_map = Arc::make_mut(&mut mental_map.0);

        for visible_pos in &visible_tiles {
            if mut_map[visible_pos.y as usize][visible_pos.x as usize].is_none() {
                if let Some(tile) = map.get_tile(visible_pos.x, visible_pos.y) {
                    mut_map[visible_pos.y as usize][visible_pos.x as usize] =
                        Some(MemoryTile { tile });
                }

                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let neighbor_x = visible_pos.x as i32 + dx;
                        let neighbor_y = visible_pos.y as i32 + dy;

                        if neighbor_x >= 0
                            && neighbor_x < map.width as i32
                            && neighbor_y >= 0
                            && neighbor_y < map.height as i32
                        {
                            let nx = neighbor_x as u32;
                            let ny = neighbor_y as u32;
                            if mut_map[ny as usize][nx as usize].is_none() {
                                let frontier_pos = Position { x: nx, y: ny };
                                if !exploration_frontier.0.contains(&frontier_pos) {
                                    exploration_frontier.0.push_back(frontier_pos);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
