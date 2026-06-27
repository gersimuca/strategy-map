use egui::{InputState, Pos2};
use crate::engine::Game;
use crate::engine::selection::{select_unit_at_pixel, deselect_all, select_in_rect};
use crate::engine::pathfinding::find_path;

pub struct InputHandler {
    pub drag_start: Option<Pos2>,
    pub drag_world_start: Option<Pos2>,
    pub is_camera_drag: bool,
    pub box_select_start: Option<Pos2>,
    pub last_scroll_pos: Option<Pos2>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            drag_start: None,
            drag_world_start: None,
            is_camera_drag: false,
            box_select_start: None,
            last_scroll_pos: None,
        }
    }
}

pub fn handle_input(game: &mut Game, input: &InputState, viewport_origin: Pos2) {
    let tile_size = 32.0;

    // Zoom with scroll
    if let Some(hover) = input.pointer.hover_pos() {
        let scroll = input.smooth_scroll_delta.y;
        if scroll.abs() > 0.1 {
            let factor = if scroll > 0.0 { 1.08 } else { 0.92 };
            // Adjust hover pos relative to viewport
            let local_pos = Pos2::new(hover.x - viewport_origin.x, hover.y - viewport_origin.y);
            game.camera.apply_zoom(factor, Some(local_pos));
        }
    }

    let pointer_pos = input.pointer.hover_pos().map(|p| {
        Pos2::new(p.x - viewport_origin.x, p.y - viewport_origin.y)
    });

    // Camera drag with right mouse or middle mouse
    if input.pointer.secondary_pressed() {
        if let Some(pos) = pointer_pos {
            game.input_handler.drag_start = Some(pos);
            game.input_handler.is_camera_drag = true;
        }
    }

    if input.pointer.secondary_released() {
        game.input_handler.is_camera_drag = false;
        game.input_handler.drag_start = None;
    }

    if game.input_handler.is_camera_drag {
        if let Some(delta) = pointer_pos {
            if let Some(last) = game.input_handler.drag_start {
                let d = egui::Vec2::new(delta.x - last.x, delta.y - last.y);
                game.camera.pan(d);
            }
            game.input_handler.drag_start = pointer_pos;
        }
    }

    // Left click handling
    if input.pointer.primary_pressed() {
        if let Some(screen_pos) = pointer_pos {
            let world_pos = game.camera.screen_to_world(screen_pos);
            let world_tile_x = (world_pos.x / tile_size) as i32;
            let world_tile_y = (world_pos.y / tile_size) as i32;

            // Convert pixel pos for unit checking (in world space)
            let hit_unit = select_unit_at_pixel(&mut game.units, world_pos, tile_size);

            if hit_unit {
                // Spawn select effect
                game.effects.spawn_select(world_pos);
            } else {
                // Check if any unit is selected and give move command
                let has_selected = game.units.iter().any(|u| u.selected);
                if has_selected {
                    // Move selected units
                    let mut paths = vec![];
                    for u in game.units.iter().filter(|u| u.selected) {
                        let path = find_path(&game.map, u.tile_pos(), (world_tile_x, world_tile_y));
                        paths.push((u.id, path));
                    }
                    for (id, path) in paths {
                        if let Some(unit) = game.units.iter_mut().find(|u| u.id == id) {
                            if !path.is_empty() {
                                unit.set_path(path);
                            }
                        }
                    }

                    // Spawn move effect at clicked world pos
                    let click_world = egui::Pos2::new(
                        world_tile_x as f32 * tile_size + tile_size / 2.0,
                        world_tile_y as f32 * tile_size + tile_size / 2.0,
                    );
                    game.effects.spawn_move_order(click_world);
                } else {
                    // Start box select
                    game.input_handler.box_select_start = Some(world_pos);
                }

                // Deselect with ctrl not held
                if !input.modifiers.ctrl {
                    if !has_selected {
                        deselect_all(&mut game.units);
                    }
                }
            }
        }
    }

    if input.pointer.primary_released() {
        if let Some(start) = game.input_handler.box_select_start.take() {
            if let Some(screen_pos) = pointer_pos {
                let end = game.camera.screen_to_world(screen_pos);
                let dist = {
                    let dx = end.x - start.x;
                    let dy = end.y - start.y;
                    (dx * dx + dy * dy).sqrt()
                };
                if dist > 8.0 {
                    select_in_rect(&mut game.units, start, end, tile_size);
                }
            }
        }
    }

    // Keyboard: Escape deselects all
    if input.key_pressed(egui::Key::Escape) {
        deselect_all(&mut game.units);
    }

    // A key to select all
    if input.key_pressed(egui::Key::A) {
        for u in game.units.iter_mut() {
            u.selected = true;
        }
    }
}