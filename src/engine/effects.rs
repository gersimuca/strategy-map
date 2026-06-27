use egui::Pos2;

#[derive(Clone, Debug)]
pub struct Particle {
    pub pos: Pos2,
    pub vel: (f32, f32),
    pub color: [u8; 4],
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

impl Particle {
    pub fn new(pos: Pos2, vel: (f32, f32), color: [u8; 4], size: f32, lifetime: f32) -> Self {
        Self { pos, vel, color, size, lifetime, max_lifetime: lifetime }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.lifetime -= dt;
        self.pos.x += self.vel.0 * dt;
        self.pos.y += self.vel.1 * dt;
        self.vel.0 *= 0.95;
        self.vel.1 *= 0.95;
        self.lifetime > 0.0
    }

    pub fn alpha_factor(&self) -> f32 {
        (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug)]
pub struct ClickIndicator {
    pub pos: Pos2,
    pub radius: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: [u8; 4],
}

impl ClickIndicator {
    pub fn new(pos: Pos2, color: [u8; 4]) -> Self {
        Self {
            pos,
            radius: 0.0,
            lifetime: 0.8,
            max_lifetime: 0.8,
            color,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.lifetime -= dt;
        let t = 1.0 - (self.lifetime / self.max_lifetime);
        self.radius = t * 30.0;
        self.lifetime > 0.0
    }

    pub fn alpha(&self) -> f32 {
        (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
    }
}

pub struct EffectsSystem {
    pub particles: Vec<Particle>,
    pub click_indicators: Vec<ClickIndicator>,
    pub time: f32,
}

impl EffectsSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            click_indicators: Vec::new(),
            time: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        self.particles.retain_mut(|p| p.update(dt));
        self.click_indicators.retain_mut(|c| c.update(dt));
    }

    pub fn spawn_move_order(&mut self, pos: Pos2) {
        self.click_indicators.push(ClickIndicator::new(pos, [100, 220, 100, 220]));

        // Spawn burst particles
        use std::f32::consts::PI;
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * 2.0 * PI;
            let speed = 40.0 + (i % 3) as f32 * 20.0;
            self.particles.push(Particle::new(
                pos,
                (angle.cos() * speed, angle.sin() * speed),
                [120, 255, 120, 200],
                3.0 + (i % 2) as f32 * 2.0,
                0.4 + (i % 3) as f32 * 0.1,
            ));
        }
    }

    pub fn spawn_select(&mut self, pos: Pos2) {
        use std::f32::consts::PI;
        for i in 0..6 {
            let angle = (i as f32 / 6.0) * 2.0 * PI;
            let speed = 30.0;
            self.particles.push(Particle::new(
                pos,
                (angle.cos() * speed, angle.sin() * speed),
                [255, 220, 60, 200],
                2.5,
                0.3,
            ));
        }
    }
}