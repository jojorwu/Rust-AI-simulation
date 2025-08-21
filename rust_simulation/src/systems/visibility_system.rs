use bevy_ecs::prelude::*;
use crate::components::{Position, BrainComponent};
use crate::map::Map;
use crate::brain::MemoryTile;
use crate::fov;
use std::collections::VecDeque;

const VISION_RADIUS: i32 = 8; // TODO: Move to config.rs

pub fn visibility_system(
    map: Res<Map>,
    mut query: Query<(&Position, &mut BrainComponent)>,
) {
    for (pos, mut brain) in query.iter_mut() {
        // 1. Calculate FOV
        let visible_tiles = fov::field_of_view(pos, VISION_RADIUS, &map);

        // 2. Update mental map and exploration frontier
        for visible_pos in visible_tiles {
            // If this tile is not yet in our mental map, it's a new discovery
            if brain.mental_map[visible_pos.y as usize][visible_pos.x as usize].is_none() {

                // Add the actual tile info to our mental map
                if let Some(tile) = map.get_tile(visible_pos.x, visible_pos.y) {
                    brain.mental_map[visible_pos.y as usize][visible_pos.x as usize] = Some(MemoryTile { tile });
                }

                // 3. Check neighbors of the new tile to update the frontier
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let neighbor_x = visible_pos.x as i32 + dx;
                        let neighbor_y = visible_pos.y as i32 + dy;

                        if neighbor_x >= 0 && neighbor_x < map.width as i32 && neighbor_y >= 0 && neighbor_y < map.height as i32 {
                            let nx = neighbor_x as u32;
                            let ny = neighbor_y as u32;
                            // If the neighbor is unknown, add it to the frontier
                            if brain.mental_map[ny as usize][nx as usize].is_none() {
                                let frontier_pos = Position { x: nx, y: ny };
                                if !brain.exploration_frontier.contains(&frontier_pos) {
                                    brain.exploration_frontier.push_back(frontier_pos);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove any frontier tiles that are now visible by building a new frontier
        let mut new_frontier = VecDeque::new();
        for p in brain.exploration_frontier.iter() {
            if brain.mental_map[p.y as usize][p.x as usize].is_none() {
                new_frontier.push_back(*p);
            }
        }
        brain.exploration_frontier = new_frontier;
    }
}
