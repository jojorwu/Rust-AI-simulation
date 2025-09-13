use crate::brain::MemoryTile;
use crate::components::{
    Position,
    ai::{ExplorationFrontier, MentalMap},
};
use crate::{config::Config, fov, map::Map};
use bevy_ecs::prelude::*;
use log::debug;
use std::sync::Arc;

pub fn visibility_system(
    map: Res<Map>,
    config: Res<Config>,
    mut query: Query<
        (Entity, &Position, &mut MentalMap, &mut ExplorationFrontier),
        Changed<Position>,
    >,
) {
    for (entity, pos, mut mental_map, mut exploration_frontier) in query.iter_mut() {
        let visible_tiles = fov::field_of_view(pos, config.ai.vision_radius, &map);
        let old_frontier_size = exploration_frontier.0.len();

        // Get a mutable reference to the HashMap inside the Arc.
        // This will create a copy if the map is currently being shared (e.g., by a pathfinding task),
        // ensuring that we don't cause data races. This is a copy-on-write strategy.
        let mental_map_mut = Arc::make_mut(&mut mental_map.0);

        for visible_pos in &visible_tiles {
            let tile_coords = (visible_pos.x, visible_pos.y);
            // If we haven't seen this tile before, add it to our mental map.
            if let std::collections::hash_map::Entry::Vacant(e) = mental_map_mut.entry(tile_coords) {
                if let Some(tile) = map.get_tile(visible_pos.x, visible_pos.y) {
                    e.insert(MemoryTile { tile });
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
                            if !mental_map_mut.contains_key(&neighbor_coords) {
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
            .retain(|p| !mental_map_mut.contains_key(&(p.x, p.y)));

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
    }
}
