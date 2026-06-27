use egui::{Pos2, Vec2};

pub struct Camera {
    pub offset: Vec2,
    pub zoom: f32,
    pub target_offset: Vec2,
    pub target_zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            offset: Vec2::new(50.0, 50.0),
            zoom: 1.5,
            target_offset: Vec2::new(50.0, 50.0),
            target_zoom: 1.5,
        }
    }

    pub fn world_to_screen(&self, p: Pos2) -> Pos2 {
        Pos2::new(
            p.x * self.zoom + self.offset.x,
            p.y * self.zoom + self.offset.y,
        )
    }

    pub fn screen_to_world(&self, p: Pos2) -> Pos2 {
        Pos2::new(
            (p.x - self.offset.x) / self.zoom,
            (p.y - self.offset.y) / self.zoom,
        )
    }

    pub fn pan(&mut self, delta: Vec2) {
        self.offset += delta;
        self.target_offset += delta;
    }

    pub fn apply_zoom(&mut self, factor: f32, anchor: Option<Pos2>) {
        let old_zoom = self.target_zoom;
        self.target_zoom = (self.target_zoom * factor).clamp(0.4, 5.0);

        // Zoom toward cursor position
        if let Some(anchor) = anchor {
            let scale = self.target_zoom / old_zoom;
            self.target_offset = Vec2::new(
                anchor.x - (anchor.x - self.target_offset.x) * scale,
                anchor.y - (anchor.y - self.target_offset.y) * scale,
            );
        }
    }

    pub fn smooth_update(&mut self, dt: f32) {
        let speed = (8.0 * dt).clamp(0.0, 1.0);
        self.zoom += (self.target_zoom - self.zoom) * speed;
        self.offset.x += (self.target_offset.x - self.offset.x) * speed;
        self.offset.y += (self.target_offset.y - self.offset.y) * speed;
    }
}