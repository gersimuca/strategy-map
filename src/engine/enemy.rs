use egui::Pos2;
use crate::engine::map::Map;
use crate::engine::unit::DamageNumber;

// Enemy type
#[derive(Clone, PartialEq, Debug)]
pub enum EnemyType {
    Grunt,
    Brute,
    Archer,
}

impl EnemyType {
    pub fn name(&self) -> &str {
        match self { Self::Grunt => "Grunt", Self::Brute => "Brute", Self::Archer => "Archer" }
    }
    pub fn speed(&self) -> f32 {
        match self { Self::Grunt => 1.8, Self::Brute => 1.1, Self::Archer => 1.6 }
    }
    pub fn max_health(&self) -> f32 {
        match self { Self::Grunt => 60.0, Self::Brute => 160.0, Self::Archer => 45.0 }
    }
    pub fn radius(&self) -> f32 {
        match self { Self::Grunt => 8.5, Self::Brute => 12.0, Self::Archer => 7.5 }
    }
    pub fn attack_range_tiles(&self) -> f32 {
        match self { Self::Grunt => 1.8, Self::Brute => 1.4, Self::Archer => 7.0 }
    }
    pub fn attack_damage(&self) -> f32 {
        match self { Self::Grunt => 12.0, Self::Brute => 28.0, Self::Archer => 8.0 }
    }
    pub fn attack_cooldown_secs(&self) -> f32 {
        match self { Self::Grunt => 1.0, Self::Brute => 1.9, Self::Archer => 1.5 }
    }
    pub fn detect_range_tiles(&self) -> f32 {
        match self { Self::Grunt => 5.5, Self::Brute => 4.0, Self::Archer => 7.5 }
    }
    pub fn visual_height(&self) -> f32 {
        match self { Self::Grunt => 4.5, Self::Brute => 8.0, Self::Archer => 4.0 }
    }
    pub fn xp_value(&self) -> u32 {
        match self { Self::Grunt => 10, Self::Brute => 30, Self::Archer => 15 }
    }
    pub fn is_ranged(&self) -> bool { matches!(self, Self::Archer) }
}

// AI state
#[derive(Clone, Debug, PartialEq)]
pub enum AiState {
    Patrol,
    Chase  { target_id: usize },
    Attack { target_id: usize },
}

// Enemy struct
#[derive(Clone, Debug)]
pub struct Enemy {
    pub id:              usize,
    pub enemy_type:      EnemyType,
    pub x:               i32,
    pub y:               i32,
    pub pixel_x:         f32,
    pub pixel_y:         f32,
    pub spawn_x:         i32,
    pub spawn_y:         i32,
    pub path:            Vec<(i32, i32)>,
    pub health:          f32,
    pub max_health:      f32,
    pub ai_state:        AiState,
    pub attack_cooldown: f32,
    pub attack_flash:    f32,
    pub damage_numbers:  Vec<DamageNumber>,
    pub patrol_timer:    f32,
    pub selection_pulse: f32,
    pub dead:            bool,
    pub death_timer:     f32,
}

impl Enemy {
    pub fn new(id: usize, x: i32, y: i32, enemy_type: EnemyType) -> Self {
        let mh = enemy_type.max_health();
        Self {
            id, enemy_type,
            x, y, spawn_x: x, spawn_y: y,
            pixel_x: x as f32 * 32.0 + 16.0,
            pixel_y: y as f32 * 32.0 + 16.0,
            path:            vec![],
            health:          mh,
            max_health:      mh,
            ai_state:        AiState::Patrol,
            attack_cooldown: 0.0,
            attack_flash:    0.0,
            damage_numbers:  vec![],
            patrol_timer:    (id as f32 * 1.37).fract() * 3.0,
            selection_pulse: 0.0,
            dead:            false,
            death_timer:     1.2,
        }
    }

    pub fn world_pos(&self) -> Pos2 { Pos2::new(self.pixel_x, self.pixel_y) }
    pub fn tile_pos(&self)  -> (i32, i32) { (self.x, self.y) }

    /// Step one frame along current path
    pub fn step(&mut self, dt: f32) {
        if self.path.is_empty() { return; }
        const TS: f32 = 32.0;
        let (tx, ty) = self.path[0];
        let tpx = tx as f32 * TS + TS / 2.0;
        let tpy = ty as f32 * TS + TS / 2.0;
        let dx  = tpx - self.pixel_x;
        let dy  = tpy - self.pixel_y;
        let dist = (dx * dx + dy * dy).sqrt();
        let step = self.enemy_type.speed() * TS * dt;
        if dist <= step {
            self.pixel_x = tpx; self.pixel_y = tpy;
            self.x = tx;        self.y = ty;
            self.path.remove(0);
        } else {
            let n = dist.max(0.001);
            self.pixel_x += dx / n * step;
            self.pixel_y += dy / n * step;
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.selection_pulse = (self.selection_pulse + dt * 3.5) % std::f32::consts::TAU;
        self.attack_flash    = (self.attack_flash - dt * 4.0).max(0.0);
        if self.attack_cooldown > 0.0 { self.attack_cooldown -= dt; }
        self.damage_numbers.retain_mut(|d| {
            d.lifetime -= dt;
            d.pos.y    -= 28.0 * dt;
            d.lifetime > 0.0
        });
        if self.dead { self.death_timer -= dt; }
    }

    pub fn take_damage(&mut self, dmg: f32) {
        let crit   = dmg > self.enemy_type.attack_damage() * 1.4;
        let actual = if crit { dmg * 1.5 } else { dmg };
        self.health -= actual;
        self.damage_numbers.push(DamageNumber {
            value:    actual,
            pos:      Pos2::new(self.pixel_x, self.pixel_y - self.enemy_type.radius() * 2.2),
            lifetime: 1.1,
            is_crit:  crit,
        });
        self.attack_flash = 1.0;
    }
}