use std::collections::{BinaryHeap, HashMap};
use crate::engine::map::{Map, MAP_W, MAP_H};

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

fn h(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

pub fn find_path(map: &Map, start: (i32, i32), goal: (i32, i32)) -> Vec<(i32, i32)> {
    let mut open = BinaryHeap::new();
    let mut came = HashMap::new();
    let mut g = HashMap::new();

    g.insert(start, 0);
    open.push(Node { cost: 0, pos: start });

    let dirs = [(1,0),(-1,0),(0,1),(0,-1)];

    while let Some(Node { pos, .. }) = open.pop() {
        if pos == goal {
            let mut path = vec![goal];
            let mut cur = goal;
            while let Some(&p) = came.get(&cur) {
                path.push(p);
                cur = p;
            }
            path.reverse();
            return path;
        }

        for d in dirs {
            let next = (pos.0 + d.0, pos.1 + d.1);
            if !map.walkable(next.0, next.1) {
                continue;
            }

            let tentative = g.get(&pos).unwrap_or(&99999) + 1;

            if tentative < *g.get(&next).unwrap_or(&99999) {
                came.insert(next, pos);
                g.insert(next, tentative);

                let f = tentative + h(next, goal);
                open.push(Node { cost: f, pos: next });
            }
        }
    }

    vec![]
}
