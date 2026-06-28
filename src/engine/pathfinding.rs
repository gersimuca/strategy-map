use std::collections::{BinaryHeap, HashMap};
use crate::engine::map::Map;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    cost: i32,
    pos:  (i32, i32),
}

impl Ord for Node {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering { o.cost.cmp(&self.cost) }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(o)) }
}

fn heuristic(a: (i32, i32), b: (i32, i32)) -> i32 {
    let dx = (a.0 - b.0).abs();
    let dy = (a.1 - b.1).abs();
    dx + dy + dx.min(dy) // octile
}

pub fn find_path(map: &Map, start: (i32, i32), goal: (i32, i32)) -> Vec<(i32, i32)> {
    if !map.walkable(goal.0, goal.1) || start == goal {
        return vec![];
    }

    let mut open = BinaryHeap::new();
    let mut came: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g:    HashMap<(i32, i32), i32>         = HashMap::new();

    g.insert(start, 0);
    open.push(Node { cost: 0, pos: start });

    const DIRS: [(i32, i32); 8] = [
        (1, 0), (-1, 0), (0, 1), (0, -1),
        (1, 1), (-1, 1), (1, -1), (-1, -1),
    ];

    while let Some(Node { pos, .. }) = open.pop() {
        if pos == goal {
            let mut path = vec![goal];
            let mut cur  = goal;
            while let Some(&p) = came.get(&cur) {
                path.push(p);
                cur = p;
                if cur == start { break; }
            }
            path.reverse();
            return path;
        }

        for d in DIRS {
            let next = (pos.0 + d.0, pos.1 + d.1);
            if !map.walkable(next.0, next.1) { continue; }

            let tc = map.get_tile(next.0, next.1)
                .map(|t| t.move_cost())
                .unwrap_or(2);
            let mc = if d.0 != 0 && d.1 != 0 { (tc as f32 * 1.41) as i32 } else { tc };

            let tg = g.get(&pos).copied().unwrap_or(99_999) + mc;
            if tg < g.get(&next).copied().unwrap_or(99_999) {
                came.insert(next, pos);
                g.insert(next, tg);
                open.push(Node { cost: tg + heuristic(next, goal), pos: next });
            }
        }
    }

    vec![]
}