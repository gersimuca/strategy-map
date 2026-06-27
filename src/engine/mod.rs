pub mod map;
pub mod unit;
pub mod selection;
pub mod input;
pub mod camera;
pub mod pathfinding;
pub mod effects;

use map::Map;
use unit::{Unit, UnitType};
use camera::Camera;
use effects::EffectsSystem;
use input::InputHandler;

pub struct Game {
    pub map: Map,
    pub units: Vec<Unit>,
    pub camera: Camera,
    pub effects: EffectsSystem,
    pub input_handler: InputHandler,
    pub time: f32,
    pub minimap_visible: bool,
    pub stats_visible: bool,
    pub selected_unit_panel: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            units: vec![
                Unit::new(0, 5, 5, UnitType::Warrior),
                Unit::new(1, 7, 5, UnitType::Archer),
                Unit::new(2, 9, 5, UnitType::Scout),
                Unit::new(3, 5, 7, UnitType::Warrior),
                Unit::new(4, 7, 7, UnitType::Archer),
            ],
            camera: Camera::new(),
            effects: EffectsSystem::new(),
            input_handler: InputHandler::new(),
            time: 0.0,
            minimap_visible: true,
            stats_visible: true,
            selected_unit_panel: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        for u in &mut self.units {
            u.update(&self.map, dt);
        }
        self.effects.update(dt);
        self.camera.smooth_update(dt);
    }

    pub fn selected_count(&self) -> usize {
        self.units.iter().filter(|u| u.selected).count()
    }

    pub fn selected_units(&self) -> Vec<&Unit> {
        self.units.iter().filter(|u| u.selected).collect()
    }
}