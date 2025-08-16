use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use super::brain::MemoryTile;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Node {
    position: (u32, u32),
    g_cost: u32,
    h_cost: u32,
    f_cost: u32,
    parent: Option<Box<Node>>,
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
    (a.0 as i32 - b.0 as i32).abs() as u32 + (a.1 as i32 - b.1 as i32).abs() as u32
}

pub fn find_path(start: (u32, u32), goal: (u32, u32), mental_map: &Vec<Vec<Option<MemoryTile>>>) -> Option<Vec<(u32, u32)>> {
    let mut open_list = BinaryHeap::new();
    let mut closed_list = HashSet::new();
    let mut g_costs = HashMap::new();

    let start_node = Node {
        position: start,
        g_cost: 0,
        h_cost: heuristic(start, goal),
        f_cost: heuristic(start, goal),
        parent: None,
    };

    open_list.push(start_node);
    g_costs.insert(start, 0);

    while let Some(current_node) = open_list.pop() {
        if current_node.position == goal {
            let mut path = Vec::new();
            let mut current = Some(Box::new(current_node));
            while let Some(node) = current {
                path.push(node.position);
                current = node.parent;
            }
            path.reverse();
            return Some(path);
        }

        closed_list.insert(current_node.position);

        for neighbor_pos in get_neighbors(current_node.position, mental_map) {
            if closed_list.contains(&neighbor_pos) {
                continue;
            }

            let tentative_g_cost = current_node.g_cost + 1;

            if tentative_g_cost < *g_costs.get(&neighbor_pos).unwrap_or(&u32::MAX) {
                g_costs.insert(neighbor_pos, tentative_g_cost);
                let h_cost = heuristic(neighbor_pos, goal);
                let neighbor_node = Node {
                    position: neighbor_pos,
                    g_cost: tentative_g_cost,
                    h_cost,
                    f_cost: tentative_g_cost + h_cost,
                    parent: Some(Box::new(current_node.clone())),
                };
                open_list.push(neighbor_node);
            }
        }
    }

    None // No path found
}

fn get_neighbors(position: (u32, u32), mental_map: &Vec<Vec<Option<MemoryTile>>>) -> Vec<(u32, u32)> {
    let mut neighbors = Vec::new();
    let (x, y) = position;

    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

    for (dx, dy) in &directions {
        let new_x = x as i32 + dx;
        let new_y = y as i32 + dy;

        if new_x >= 0 && new_x < mental_map[0].len() as i32 && new_y >= 0 && new_y < mental_map.len() as i32 {
            let new_pos = (new_x as u32, new_y as u32);
            if let Some(Some(memory_tile)) = mental_map.get(new_y as usize).and_then(|row| row.get(new_x as usize)) {
                if memory_tile.tile.tile_type == '.' || memory_tile.tile.tile_type == 'S' { // Allow walking on empty ground and sand
                    neighbors.push(new_pos);
                }
            }
        }
    }

    neighbors
}
