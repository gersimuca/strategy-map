use std::collections::{BinaryHeap, HashMap};
use crate::engine::map::Map;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    cost: i32,
    pos: (i32, i32),
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(a: (i32, i32), b: (i32, i32)) -> i32 {
    // Diagonal distance heuristic
    let dx = (a.0 - b.0).abs();
    let dy = (a.1 - b.1).abs();
    (dx + dy) + (dx.min(dy)) // slightly prefer diagonal
}

pub fn find_path(map: &Map, start: (i32, i32), goal: (i32, i32)) -> Vec<(i32, i32)> {
    if !map.walkable(goal.0, goal.1) {
        return vec![];
    }
    if start == goal {
        return vec![];
    }

    let mut open = BinaryHeap::new();
    let mut came: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g: HashMap<(i32, i32), i32> = HashMap::new();

    g.insert(start, 0);
    open.push(Node { cost: 0, pos: start });

    // 8-directional movement
    let dirs = [
        (1, 0), (-1, 0), (0, 1), (0, -1),
        (1, 1), (-1, 1), (1, -1), (-1, -1),
    ];

    while let Some(Node { pos, .. }) = open.pop() {
        if pos == goal {
            let mut path = vec![goal];
            let mut cur = goal;
            while let Some(&p) = came.get(&cur) {
                path.push(p);
                cur = p;
                if cur == start {
                    break;
                }
            }
            path.reverse();
            return path;
        }

        for d in dirs {
            let next = (pos.0 + d.0, pos.1 + d.1);
            if !map.walkable(next.0, next.1) {
                continue;
            }

            // Get tile cost
            let tile_cost = map.get_tile(next.0, next.1)
                .map(|t| t.move_cost())
                .unwrap_or(2);

            // Diagonal moves cost a bit more
            let move_cost = if d.0 != 0 && d.1 != 0 {
                (tile_cost as f32 * 1.41) as i32
            } else {
                tile_cost
            };

            let tentative = g.get(&pos).unwrap_or(&99999) + move_cost;
            if tentative < *g.get(&next).unwrap_or(&99999) {
                came.insert(next, pos);
                g.insert(next, tentative);
                let f = tentative + heuristic(next, goal);
                open.push(Node { cost: f, pos: next });
            }
        }
    }

    vec![]
}