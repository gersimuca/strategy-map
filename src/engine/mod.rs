pub mod map;
pub mod unit;
pub mod selection;
pub mod input;
pub mod camera;
pub mod pathfinding;

use map::Map;
use unit::Unit;
use camera::Camera;

pub struct Game {
    pub map: Map,
    pub units: Vec<Unit>,
    pub camera: Camera,
}

impl Game {
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            units: vec![
                Unit::new(5, 5),
                Unit::new(10, 10),
            ],
            camera: Camera::new(),
        }
    }

    pub fn update(&mut self) {
        for u in &mut self.units {
            u.update(&self.map);
        }
    }
}
