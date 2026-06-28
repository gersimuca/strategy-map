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
        match self { Self::Warrior => "Warrior", Self::Archer => "Archer", Self::Scout => "Scout" }
    }
    pub fn speed(&self) -> f32 {
        match self { Self::Warrior => 2.2, Self::Archer => 2.8, Self::Scout => 5.0 }
    }
    pub fn max_health(&self) -> f32 {
        match self { Self::Warrior => 120.0, Self::Archer => 70.0, Self::Scout => 90.0 }
    }
    pub fn radius(&self) -> f32 {
        match self { Self::Warrior => 9.5, Self::Archer => 7.5, Self::Scout => 8.0 }
    }
    pub fn attack_range_tiles(&self) -> f32 {
        match self { Self::Warrior => 1.6, Self::Archer => 6.0, Self::Scout => 2.0 }
    }
    pub fn attack_damage(&self) -> f32 {
        match self { Self::Warrior => 18.0, Self::Archer => 10.0, Self::Scout => 8.0 }
    }
    pub fn attack_cooldown_secs(&self) -> f32 {
        match self { Self::Warrior => 1.1, Self::Archer => 1.6, Self::Scout => 0.85 }
    }
    /// Extra upward pixel offset for pseudo-3-D raised look
    pub fn visual_height(&self) -> f32 {
        match self { Self::Warrior => 7.0, Self::Archer => 4.5, Self::Scout => 3.5 }
    }
}

// Floating damage number
#[derive(Clone, Debug)]
pub struct DamageNumber {
    pub value:    f32,
    pub pos:      Pos2,
    pub lifetime: f32,
    pub is_crit:  bool,
}

// Move trail crumb
#[derive(Clone, Debug)]
pub struct MoveEffect {
    pub pos:      Pos2,
    pub alpha:    f32,
    pub lifetime: f32,
}

// The unit
#[derive(Clone, Debug)]
pub struct Unit {
    pub id:              usize,
    pub unit_type:       UnitType,
    pub x:               i32,
    pub y:               i32,
    pub pixel_x:         f32,
    pub pixel_y:         f32,
    pub path:            Vec<(i32, i32)>,
    pub selected:        bool,
    pub health:          f32,
    pub max_health:      f32,
    pub move_effects:    Vec<MoveEffect>,
    pub selection_pulse: f32,
    pub is_moving:       bool,
    // combat
    pub attack_cooldown: f32,
    pub attacking:       bool,
    pub attack_target:   Option<usize>,
    pub damage_numbers:  Vec<DamageNumber>,
    pub attack_flash:    f32,
    pub dead:            bool,
}

impl Unit {
    pub fn new(id: usize, x: i32, y: i32, unit_type: UnitType) -> Self {
        let mh = unit_type.max_health();
        Self {
            id, unit_type,
            x, y,
            pixel_x: x as f32 * 32.0 + 16.0,
            pixel_y: y as f32 * 32.0 + 16.0,
            path:            vec![],
            selected:        false,
            health:          mh,
            max_health:      mh,
            move_effects:    vec![],
            selection_pulse: 0.0,
            is_moving:       false,
            attack_cooldown: 0.0,
            attacking:       false,
            attack_target:   None,
            damage_numbers:  vec![],
            attack_flash:    0.0,
            dead:            false,
        }
    }

    pub fn update(&mut self, map: &Map, dt: f32) {
        if self.dead { return; }
        const TS: f32 = 32.0;

        self.selection_pulse = (self.selection_pulse + dt * 3.5) % (std::f32::consts::TAU);
        self.attack_flash    = (self.attack_flash - dt * 4.0).max(0.0);
        if self.attack_cooldown > 0.0 { self.attack_cooldown -= dt; }

        // Trail crumbs
        self.move_effects.retain_mut(|e| {
            e.lifetime -= dt;
            e.alpha     = (e.lifetime / 0.45).clamp(0.0, 1.0);
            e.lifetime > 0.0
        });

        // Floating damage numbers
        self.damage_numbers.retain_mut(|d| {
            d.lifetime -= dt;
            d.pos.y    -= 28.0 * dt;
            d.lifetime > 0.0
        });

        // Movement (stopped while attacking)
        if !self.path.is_empty() && !self.attacking {
            let (tx, ty) = self.path[0];
            let tpx = tx as f32 * TS + TS / 2.0;
            let tpy = ty as f32 * TS + TS / 2.0;
            let dx  = tpx - self.pixel_x;
            let dy  = tpy - self.pixel_y;
            let dist = (dx * dx + dy * dy).sqrt();
            let step = self.unit_type.speed() * TS * dt;

            if dist <= step {
                self.pixel_x   = tpx;
                self.pixel_y   = tpy;
                self.x         = tx;
                self.y         = ty;
                self.path.remove(0);
                self.is_moving = !self.path.is_empty();
            } else {
                // Push trail crumb
                let push = self.move_effects.is_empty() || {
                    let l = self.move_effects.last().unwrap();
                    ((l.pos.x - self.pixel_x).powi(2) + (l.pos.y - self.pixel_y).powi(2)).sqrt() > 18.0
                };
                if push {
                    self.move_effects.push(MoveEffect {
                        pos:      Pos2::new(self.pixel_x, self.pixel_y),
                        alpha:    0.5,
                        lifetime: 0.45,
                    });
                }
                let n = dist.max(0.001);
                self.pixel_x += dx / n * step;
                self.pixel_y += dy / n * step;
                self.is_moving = true;
            }
        } else if !self.attacking {
            self.is_moving = false;
        }
    }

    pub fn set_path(&mut self, path: Vec<(i32, i32)>) {
        self.path          = if path.len() > 1 { path[1..].to_vec() } else { path };
        self.attacking     = false;
        self.attack_target = None;
    }

    pub fn world_pos(&self) -> Pos2 { Pos2::new(self.pixel_x, self.pixel_y) }
    pub fn tile_pos(&self)  -> (i32, i32) { (self.x, self.y) }

    pub fn take_damage(&mut self, dmg: f32) {
        let crit = rand_bool(dmg * 0.08);
        let actual = if crit { dmg * 1.6 } else { dmg };
        self.health -= actual;
        self.damage_numbers.push(DamageNumber {
            value:    actual,
            pos:      Pos2::new(self.pixel_x, self.pixel_y - self.unit_type.radius() * 2.0),
            lifetime: 1.1,
            is_crit:  crit,
        });
        self.attack_flash = 1.0;
    }
}

fn rand_bool(p: f32) -> bool {
    // cheap LCG-style pseudo-random without importing rand
    use std::time::SystemTime;
    let t = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(12345) as f32;
    (t % 100.0) < (p * 100.0)
}