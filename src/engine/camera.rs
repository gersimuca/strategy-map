use egui::Vec2;

pub struct Camera {
    pub offset: Vec2,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            offset: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }

    pub fn world_to_screen(&self, p: egui::Pos2) -> egui::Pos2 {
        p * self.zoom + self.offset
    }

    pub fn screen_to_world(&self, p: egui::Pos2) -> egui::Pos2 {
        (p.to_vec2() - self.offset) / self.zoom
    }

    pub fn pan(&mut self, delta: Vec2) {
        self.offset += delta;
    }

    pub fn apply_zoom(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.3, 4.0);
    }
}
