use egui::Pos2;
use crate::engine::unit::Unit;

pub fn select_unit_at_pixel(units: &mut [Unit], pos: Pos2, tile_size: f32) -> bool {
    let radius = 12.0 * tile_size / 32.0 * 1.5; // screen-space threshold
    for u in units.iter_mut() {
        let ux = u.pixel_x;
        let uy = u.pixel_y;
        let dx = ux - pos.x;
        let dy = uy - pos.y;
        if (dx * dx + dy * dy).sqrt() < radius {
            u.selected = !u.selected;
            return true;
        }
    }
    false
}

pub fn select_unit_at_tile(units: &mut [Unit], x: i32, y: i32) -> bool {
    for u in units.iter_mut() {
        if u.x == x && u.y == y {
            u.selected = !u.selected;
            return true;
        }
    }
    false
}

pub fn deselect_all(units: &mut [Unit]) {
    for u in units.iter_mut() {
        u.selected = false;
    }
}

pub fn select_in_rect(units: &mut [Unit], rect_min: Pos2, rect_max: Pos2, tile_size: f32) {
    let min_x = rect_min.x.min(rect_max.x);
    let max_x = rect_min.x.max(rect_max.x);
    let min_y = rect_min.y.min(rect_max.y);
    let max_y = rect_min.y.max(rect_max.y);

    for u in units.iter_mut() {
        let ux = u.pixel_x;
        let uy = u.pixel_y;
        if ux >= min_x && ux <= max_x && uy >= min_y && uy <= max_y {
            u.selected = true;
        }
    }
}