use crate::engine::map::Map;
use crate::engine::pathfinding::step_path;

#[derive(Clone)]
pub struct Unit {
    pub x: i32,
    pub y: i32,
    pub path: Vec<(i32, i32)>,
    pub selected: bool,
}

impl Unit {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            path: vec![],
            selected: false,
        }
    }

    pub fn update(&mut self, map: &Map) {
        if !self.path.is_empty() {
            let next = self.path.remove(0);
            if map.walkable(next.0, next.1) {
                self.x = next.0;
                self.y = next.1;
            }
        }
    }

    pub fn set_path(&mut self, path: Vec<(i32, i32)>) {
        self.path = path;
    }
}
