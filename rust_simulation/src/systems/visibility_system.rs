use crate::brain::MemoryTile;
use crate::components::{
    Position,
    ai::{ExplorationFrontier, MentalMap},
};
use crate::fov;
use crate::map::Map;
use bevy_ecs::prelude::*;
use log::debug;
use rayon::prelude::*;
use std::collections::VecDeque;

const VISION_RADIUS: i32 = 8; // TODO: Move to config.rs

pub fn visibility_system(
    map: Res<Map>,
    mut query: Query<(Entity, &Position, &mut MentalMap, &mut ExplorationFrontier)>,
) {
    query
        .par_iter_mut()
        .for_each(|(entity, pos, mut mental_map, mut exploration_frontier)| {
            let visible_tiles = fov::field_of_view(pos, VISION_RADIUS, &map);
        let old_frontier_size = exploration_frontier.0.len();

        for visible_pos in &visible_tiles {
            let tile_coords = (visible_pos.x, visible_pos.y);
            // If we haven't seen this tile before, add it to our mental map.
            if !mental_map.0.contains_key(&tile_coords) {
                if let Some(tile) = map.get_tile(visible_pos.x, visible_pos.y) {
                    mental_map.0.insert(tile_coords, MemoryTile { tile });
                }

                // Check neighbors of the newly visible tile to add them to the exploration frontier.
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
                            let neighbor_coords = (nx, ny);
                            // If we haven't seen the neighbor, it's a candidate for the frontier.
                            if !mental_map.0.contains_key(&neighbor_coords) {
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

        // Prune the exploration frontier of any tiles that have now been seen.
        exploration_frontier
            .0
            .retain(|p| !mental_map.0.contains_key(&(p.x, p.y)));

        let new_frontiers = exploration_frontier
            .0
            .len()
            .saturating_sub(old_frontier_size);
        if new_frontiers > 0 {
            debug!(
                "Entity {:?} saw {} tiles, discovered {} new frontier tiles. Total frontier size: {}",
                entity,
                visible_tiles.len(),
                new_frontiers,
                exploration_frontier.0.len()
            );
        }
    });
}
