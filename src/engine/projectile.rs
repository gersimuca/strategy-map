use egui::Pos2;

#[derive(Clone, Debug, PartialEq)]
pub enum ProjOwner { Player, Enemy }

#[derive(Clone, Debug)]
pub struct Projectile {
    pub id:       usize,
    pub owner:    ProjOwner,
    pub src_id:   usize,
    pub dst_id:   usize,
    pub pos:      Pos2,
    pub target:   Pos2,
    pub vx:       f32,
    pub vy:       f32,
    pub speed:    f32,
    pub damage:   f32,
    pub radius:   f32,
    pub trail:    Vec<(Pos2, f32)>,
    pub hit:      bool,
    pub lifetime: f32,
}

impl Projectile {
    pub fn new(
        id: usize, owner: ProjOwner, src_id: usize, dst_id: usize,
        from: Pos2, to: Pos2, speed: f32, damage: f32, radius: f32,
    ) -> Self {
        let dx   = to.x - from.x;
        let dy   = to.y - from.y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);
        Self {
            id, owner, src_id, dst_id,
            pos: from, target: to,
            vx: dx / dist * speed,
            vy: dy / dist * speed,
            speed, damage, radius,
            trail:    vec![],
            hit:      false,
            lifetime: 3.5,
        }
    }

    /// Returns true when the projectile should be removed (hit or expired).
    pub fn update(&mut self, dt: f32, live_target: Option<Pos2>) -> bool {
        self.lifetime -= dt;
        if self.lifetime <= 0.0 { self.hit = true; return true; }

        // Soft homing toward live target
        if let Some(tp) = live_target {
            self.target = tp;
            let dx    = tp.x - self.pos.x;
            let dy    = tp.y - self.pos.y;
            let dist  = (dx * dx + dy * dy).sqrt().max(0.001);
            let stx   = dx / dist * self.speed;
            let sty   = dy / dist * self.speed;
            let blend = (dt * 6.0).min(1.0);
            self.vx  += (stx - self.vx) * blend;
            self.vy  += (sty - self.vy) * blend;
        }

        // Renormalise speed
        let spd  = (self.vx * self.vx + self.vy * self.vy).sqrt().max(0.001);
        self.vx  = self.vx / spd * self.speed;
        self.vy  = self.vy / spd * self.speed;

        self.pos.x += self.vx * dt;
        self.pos.y += self.vy * dt;

        // Trail
        let push = self.trail.is_empty() || {
            let (lp, _) = self.trail.last().unwrap();
            ((lp.x - self.pos.x).powi(2) + (lp.y - self.pos.y).powi(2)).sqrt() > 8.0
        };
        if push { self.trail.push((self.pos, 1.0)); }
        self.trail.retain_mut(|(_, a)| { *a -= dt * 4.0; *a > 0.0 });
        if self.trail.len() > 14 { self.trail.remove(0); }

        // Hit check
        let dx  = self.target.x - self.pos.x;
        let dy  = self.target.y - self.pos.y;
        let hit = (dx * dx + dy * dy).sqrt() < 12.0;
        if hit { self.hit = true; }
        hit
    }
}