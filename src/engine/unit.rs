use egui::Pos2;
use crate::engine::map::Map;

#[derive(Clone, PartialEq, Debug)]
pub enum UnitType {
    Warrior,
    Archer,
    Scout,
}

impl UnitType {
    pub fn name(&self) -> &str {
        match self {
            UnitType::Warrior => "Warrior",
            UnitType::Archer => "Archer",
            UnitType::Scout => "Scout",
        }
    }

    pub fn speed(&self) -> f32 {
        match self {
            UnitType::Warrior => 2.5,
            UnitType::Archer => 2.0,
            UnitType::Scout => 4.5,
        }
    }

    pub fn max_health(&self) -> f32 {
        match self {
            UnitType::Warrior => 120.0,
            UnitType::Archer => 70.0,
            UnitType::Scout => 90.0,
        }
    }

    pub fn radius(&self) -> f32 {
        match self {
            UnitType::Warrior => 9.0,
            UnitType::Archer => 7.5,
            UnitType::Scout => 8.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MoveEffect {
    pub pos: Pos2,
    pub alpha: f32,
    pub lifetime: f32,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub id: usize,
    pub unit_type: UnitType,
    pub x: i32,
    pub y: i32,
    pub pixel_x: f32,
    pub pixel_y: f32,
    pub path: Vec<(i32, i32)>,
    pub selected: bool,
    pub health: f32,
    pub max_health: f32,
    pub move_progress: f32,
    pub move_effects: Vec<MoveEffect>,
    pub selection_pulse: f32,
    pub is_moving: bool,
}

impl Unit {
    pub fn new(id: usize, x: i32, y: i32, unit_type: UnitType) -> Self {
        let max_health = unit_type.max_health();
        let px = x as f32 * 32.0 + 16.0;
        let py = y as f32 * 32.0 + 16.0;
        Self {
            id,
            unit_type,
            x,
            y,
            pixel_x: px,
            pixel_y: py,
            path: vec![],
            selected: false,
            health: max_health,
            max_health,
            move_progress: 0.0,
            move_effects: vec![],
            selection_pulse: 0.0,
            is_moving: false,
        }
    }

    pub fn update(&mut self, map: &Map, dt: f32) {
        let tile_size = 32.0;
        let speed = self.unit_type.speed();

        self.selection_pulse = (self.selection_pulse + dt * 3.0) % (std::f32::consts::PI * 2.0);

        // Update move effects
        self.move_effects.retain_mut(|e| {
            e.lifetime -= dt;
            e.alpha = (e.lifetime / 0.5).clamp(0.0, 1.0);
            e.lifetime > 0.0
        });

        if !self.path.is_empty() {
            let target = self.path[0];
            let target_px = target.0 as f32 * tile_size + tile_size / 2.0;
            let target_py = target.1 as f32 * tile_size + tile_size / 2.0;

            let dx = target_px - self.pixel_x;
            let dy = target_py - self.pixel_y;
            let dist = (dx * dx + dy * dy).sqrt();

            let step = speed * tile_size * dt;

            if dist <= step {
                self.pixel_x = target_px;
                self.pixel_y = target_py;
                self.x = target.0;
                self.y = target.1;
                self.path.remove(0);
                self.is_moving = !self.path.is_empty();
            } else {
                // Add move trail effect periodically
                if (self.pixel_x - target_px).abs() > 8.0 || (self.pixel_y - target_py).abs() > 8.0 {
                    if self.move_effects.is_empty() || {
                        let last = self.move_effects.last().unwrap();
                        let ex = last.pos.x - self.pixel_x;
                        let ey = last.pos.y - self.pixel_y;
                        (ex * ex + ey * ey).sqrt() > 20.0
                    } {
                        self.move_effects.push(MoveEffect {
                            pos: Pos2::new(self.pixel_x, self.pixel_y),
                            alpha: 0.6,
                            lifetime: 0.5,
                        });
                    }
                }

                let norm = dist.max(0.001);
                self.pixel_x += dx / norm * step;
                self.pixel_y += dy / norm * step;
                self.is_moving = true;
            }
        } else {
            self.is_moving = false;
        }
    }

    pub fn set_path(&mut self, path: Vec<(i32, i32)>) {
        if path.len() > 1 {
            self.path = path[1..].to_vec();
        } else {
            self.path = path;
        }
    }

    pub fn world_pos(&self) -> Pos2 {
        Pos2::new(self.pixel_x, self.pixel_y)
    }

    pub fn tile_pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

// Health initialization
impl Unit {
    pub fn with_health(mut self, h: f32) -> Self {
        self.health = h;
        self
    }
}