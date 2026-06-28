use egui::Pos2;

#[derive(Clone, Debug)]
pub struct Particle {
    pub pos:          Pos2,
    pub vel:          (f32, f32),
    pub color:        [u8; 4],
    pub size:         f32,
    pub lifetime:     f32,
    pub max_lifetime: f32,
}

impl Particle {
    pub fn new(pos: Pos2, vel: (f32, f32), color: [u8; 4], size: f32, lifetime: f32) -> Self {
        Self { pos, vel, color, size, lifetime, max_lifetime: lifetime }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.lifetime -= dt;
        self.pos.x    += self.vel.0 * dt;
        self.pos.y    += self.vel.1 * dt;
        self.vel.0    *= 0.92;
        self.vel.1    *= 0.92;
        self.lifetime > 0.0
    }

    pub fn alpha_factor(&self) -> f32 {
        (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug)]
pub struct ClickIndicator {
    pub pos:          Pos2,
    pub radius:       f32,
    pub lifetime:     f32,
    pub max_lifetime: f32,
    pub color:        [u8; 4],
}

impl ClickIndicator {
    pub fn new(pos: Pos2, color: [u8; 4]) -> Self {
        Self { pos, radius: 0.0, lifetime: 0.75, max_lifetime: 0.75, color }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.lifetime -= dt;
        let t         = 1.0 - self.lifetime / self.max_lifetime;
        self.radius   = t * 30.0;
        self.lifetime > 0.0
    }

    pub fn alpha(&self) -> f32 { (self.lifetime / self.max_lifetime).clamp(0.0, 1.0) }
}

pub struct EffectsSystem {
    pub particles:        Vec<Particle>,
    pub click_indicators: Vec<ClickIndicator>,
    pub time:             f32,
}

impl EffectsSystem {
    pub fn new() -> Self {
        Self { particles: vec![], click_indicators: vec![], time: 0.0 }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        self.particles.retain_mut(|p| p.update(dt));
        self.click_indicators.retain_mut(|c| c.update(dt));
    }

    pub fn spawn_move_order(&mut self, pos: Pos2) {
        self.click_indicators.push(ClickIndicator::new(pos, [100, 220, 100, 220]));
        use std::f32::consts::TAU;
        for i in 0..10u8 {
            let a  = (i as f32 / 10.0) * TAU;
            let sp = 50.0 + (i % 3) as f32 * 25.0;
            self.particles.push(Particle::new(
                pos, (a.cos() * sp, a.sin() * sp),
                [80, 220, 100, 200], 3.5, 0.5,
            ));
        }
    }

    pub fn spawn_select(&mut self, pos: Pos2) {
        use std::f32::consts::TAU;
        for i in 0..8u8 {
            let a = (i as f32 / 8.0) * TAU;
            self.particles.push(Particle::new(
                pos, (a.cos() * 30.0, a.sin() * 30.0),
                [255, 215, 60, 200], 2.5, 0.35,
            ));
        }
    }

    pub fn spawn_hit(&mut self, pos: Pos2, is_enemy_hit: bool) {
        let col: [u8; 4] = if is_enemy_hit { [200, 40, 40, 230] } else { [220, 120, 40, 230] };
        use std::f32::consts::TAU;
        for i in 0..12u8 {
            let a  = (i as f32 / 12.0) * TAU;
            let sp = 35.0 + (i % 4) as f32 * 20.0;
            self.particles.push(Particle::new(
                pos, (a.cos() * sp, a.sin() * sp),
                col, 2.0 + (i % 3) as f32, 0.4,
            ));
        }
        self.click_indicators.push(ClickIndicator::new(pos, col));
    }

    pub fn spawn_death(&mut self, pos: Pos2, is_enemy: bool) {
        let col: [u8; 4] = if is_enemy { [180, 30, 30, 245] } else { [60, 120, 220, 245] };
        use std::f32::consts::TAU;
        for i in 0..20u8 {
            let a  = (i as f32 / 20.0) * TAU;
            let sp = 40.0 + (i % 5) as f32 * 30.0;
            self.particles.push(Particle::new(
                pos, (a.cos() * sp, a.sin() * sp),
                col, 4.0, 0.85,
            ));
        }
        self.click_indicators.push(ClickIndicator::new(pos, col));
    }
}