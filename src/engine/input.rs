use egui::InputState;
use crate::engine::Game;
use crate::engine::selection::select_unit;
use crate::engine::pathfinding::find_path;

pub fn handle_input(game: &mut Game, input: &InputState) {
    let pointer = input.pointer.hover_pos();
    let clicked = input.pointer.primary_clicked();

    if let Some(pos) = pointer {
        let world_x = (pos.x / 32.0) as i32;
        let world_y = (pos.y / 32.0) as i32;

        if clicked {
            if !select_unit(&mut game.units, world_x, world_y) {
                let selected: Vec<_> = game.units
                    .iter_mut()
                    .filter(|u| u.selected)
                    .collect();

                for u in selected {
                    u.set_path(find_path(&game.map, (u.x, u.y), (world_x, world_y)));
                }
            }
        }
    }
}