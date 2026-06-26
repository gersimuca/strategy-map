use crate::engine::unit::Unit;

pub fn select_unit(units: &mut [Unit], x: i32, y: i32) -> bool {
    for u in units.iter_mut() {
        if u.x == x && u.y == y {
            u.selected = !u.selected;
            return true;
        }
    }
    false
}

pub fn get_selected(units: &mut [Unit]) -> Vec<&mut Unit> {
    units.iter_mut().filter(|u| u.selected).collect()
}
