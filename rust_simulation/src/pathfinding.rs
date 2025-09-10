use super::brain::MemoryTile;
use log::debug;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Clone, Eq, PartialEq)]
struct Node {
    position: (u32, u32),
    f_cost: u32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost.cmp(&self.f_cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(a: (u32, u32), b: (u32, u32)) -> u32 {
    (a.0 as i32 - b.0 as i32).unsigned_abs() + (a.1 as i32 - b.1 as i32).unsigned_abs()
}

pub fn find_path(
    start: (u32, u32),
    goal: (u32, u32),
    mental_map: &HashMap<(u32, u32), MemoryTile>,
    width: u32,
    height: u32,
) -> Option<Vec<(u32, u32)>> {
    let mut open_list = BinaryHeap::new();
    let mut came_from: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
    let mut g_costs = HashMap::new();

    g_costs.insert(start, 0);

    open_list.push(Node {
        position: start,
        f_cost: heuristic(start, goal),
    });

    while let Some(current_node) = open_list.pop() {
        if current_node.position == goal {
            let mut path = Vec::new();
            let mut current = goal;
            while current != start {
                path.push(current);
                current = came_from[&current];
            }
            path.push(start);
            path.reverse();
            debug!("Path found from {start:?} to {goal:?}: {path:?}");
            return Some(path);
        }

        for neighbor_pos in get_neighbors(current_node.position, mental_map, width, height) {
            let tentative_g_cost = g_costs.get(&current_node.position).unwrap_or(&u32::MAX) + 1;

            if tentative_g_cost < *g_costs.get(&neighbor_pos).unwrap_or(&u32::MAX) {
                came_from.insert(neighbor_pos, current_node.position);
                g_costs.insert(neighbor_pos, tentative_g_cost);
                let f_cost = tentative_g_cost + heuristic(neighbor_pos, goal);
                open_list.push(Node {
                    position: neighbor_pos,
                    f_cost,
                });
            }
        }
    }

    debug!("No path found from {start:?} to {goal:?}");
    None // No path found
}

fn get_neighbors(
    position: (u32, u32),
    mental_map: &HashMap<(u32, u32), MemoryTile>,
    width: u32,
    height: u32,
) -> Vec<(u32, u32)> {
    let mut neighbors = Vec::new();
    let (x, y) = position;

    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

    for (dx, dy) in &directions {
        let new_x = x as i32 + dx;
        let new_y = y as i32 + dy;

        // Check map boundaries
        if new_x >= 0 && new_x < width as i32 && new_y >= 0 && new_y < height as i32 {
            let new_pos = (new_x as u32, new_y as u32);

            // Check if the tile is known and impassable.
            // If the tile is not in the mental map, it's considered unknown and therefore walkable.
            let tile_is_known_impassable =
                if let Some(memory_tile) = mental_map.get(&new_pos) {
                    // Tile is known, check if it's a wall type
                    matches!(memory_tile.tile.tile_type, '#' | 'T' | 'M')
                } else {
                    // Tile is unknown, so it's not known to be impassable.
                    false
                };

            if !tile_is_known_impassable {
                neighbors.push(new_pos);
            }
        }
    }

    neighbors
}
