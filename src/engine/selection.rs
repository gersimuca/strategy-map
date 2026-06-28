use egui::Pos2;
use crate::engine::unit::Unit;

pub fn select_unit_at_pixel(units: &mut [Unit], pos: Pos2, zoom: f32) -> bool {
    let threshold = 13.0 * zoom.clamp(0.5, 2.0);
    for u in units.iter_mut() {
        let dx = u.pixel_x - pos.x;
        let dy = u.pixel_y - pos.y;
        if (dx * dx + dy * dy).sqrt() < threshold {
            u.selected = !u.selected;
            return true;
        }
    }
    false
}

pub fn deselect_all(units: &mut [Unit]) {
    for u in units.iter_mut() { u.selected = false; }
}

pub fn select_in_rect(units: &mut [Unit], rect_min: Pos2, rect_max: Pos2) {
    let min_x = rect_min.x.min(rect_max.x);
    let max_x = rect_min.x.max(rect_max.x);
    let min_y = rect_min.y.min(rect_max.y);
    let max_y = rect_min.y.max(rect_max.y);
    for u in units.iter_mut() {
        if u.pixel_x >= min_x && u.pixel_x <= max_x &&
            u.pixel_y >= min_y && u.pixel_y <= max_y {
            u.selected = true;
        }
    }
}