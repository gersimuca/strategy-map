use egui::{InputState, Pos2};
use crate::engine::Game;
use crate::engine::selection::{select_unit_at_pixel, deselect_all, select_in_rect};
use crate::engine::pathfinding::find_path;

pub struct InputHandler {
    pub drag_start:       Option<Pos2>,
    pub is_camera_drag:   bool,
    pub box_select_start: Option<Pos2>,  // world-space
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            drag_start:       None,
            is_camera_drag:   false,
            box_select_start: None,
        }
    }
}

pub fn handle_input(game: &mut Game, input: &InputState, viewport_origin: Pos2) {
    const TS: f32 = 32.0;

    // Scroll-wheel zoom
    if let Some(hover) = input.pointer.hover_pos() {
        let scroll = input.smooth_scroll_delta.y;
        if scroll.abs() > 0.1 {
            let factor = if scroll > 0.0 { 1.09 } else { 0.91 };
            let local  = Pos2::new(hover.x - viewport_origin.x, hover.y - viewport_origin.y);
            game.camera.apply_zoom(factor, Some(local));
        }
    }

    let pointer_local = input.pointer.hover_pos().map(|p| {
        Pos2::new(p.x - viewport_origin.x, p.y - viewport_origin.y)
    });

    // Right-drag camera pan
    if input.pointer.secondary_pressed() {
        game.input_handler.drag_start     = pointer_local;
        game.input_handler.is_camera_drag = true;
    }
    if input.pointer.secondary_released() {
        game.input_handler.is_camera_drag = false;
        game.input_handler.drag_start     = None;
    }
    if game.input_handler.is_camera_drag {
        if let Some(cur) = pointer_local {
            if let Some(last) = game.input_handler.drag_start {
                let d = egui::Vec2::new(cur.x - last.x, cur.y - last.y);
                game.camera.pan(d);
            }
            game.input_handler.drag_start = pointer_local;
        }
    }

    // Left-click
    if input.pointer.primary_pressed() {
        if let Some(screen) = pointer_local {
            let world  = game.camera.screen_to_world(screen);
            let tx     = (world.x / TS) as i32;
            let ty     = (world.y / TS) as i32;

            let hit = select_unit_at_pixel(&mut game.units, world, game.camera.zoom);

            if hit {
                game.effects.spawn_select(world);
            } else {
                let has_sel = game.units.iter().any(|u| u.selected);
                if has_sel {
                    // Formation move: spread selected units around the goal tile
                    let sel_ids: Vec<usize> = game.units.iter()
                        .filter(|u| u.selected)
                        .map(|u| u.id)
                        .collect();
                    let n = sel_ids.len();

                    for (i, &uid) in sel_ids.iter().enumerate() {
                        // Offset from center of group in a small grid
                        let off_x = if n > 1 { (i as i32 % 3) - 1 } else { 0 };
                        let off_y = if n > 1 { i as i32 / 3 }       else { 0 };
                        let gtx   = (tx + off_x).clamp(1, crate::engine::map::MAP_W - 2);
                        let gty   = (ty + off_y).clamp(1, crate::engine::map::MAP_H - 2);

                        if let Some(unit) = game.units.iter_mut().find(|u| u.id == uid) {
                            let goal  = if game.map.walkable(gtx, gty) { (gtx, gty) } else { (tx, ty) };
                            let path  = find_path(&game.map, unit.tile_pos(), goal);
                            if !path.is_empty() { unit.set_path(path); }
                        }
                    }

                    let click_world = Pos2::new(tx as f32 * TS + TS / 2.0, ty as f32 * TS + TS / 2.0);
                    game.effects.spawn_move_order(click_world);
                } else {
                    // Start box-select
                    if !input.modifiers.ctrl { deselect_all(&mut game.units); }
                    game.input_handler.box_select_start = Some(world);
                }
            }
        }
    }

    // Release: finish box-select
    if input.pointer.primary_released() {
        if let Some(start) = game.input_handler.box_select_start.take() {
            if let Some(screen) = pointer_local {
                let end = game.camera.screen_to_world(screen);
                let dx  = end.x - start.x;
                let dy  = end.y - start.y;
                if (dx * dx + dy * dy).sqrt() > 10.0 {
                    select_in_rect(&mut game.units, start, end);
                }
            }
        }
    }

    // Keyboard
    if input.key_pressed(egui::Key::Escape) {
        deselect_all(&mut game.units);
    }
    if input.key_pressed(egui::Key::A) {
        for u in &mut game.units { u.selected = true; }
    }
    // S = stop selected units
    if input.key_pressed(egui::Key::S) {
        for u in game.units.iter_mut().filter(|u| u.selected) {
            u.path.clear();
            u.attacking     = false;
            u.attack_target = None;
        }
    }
}