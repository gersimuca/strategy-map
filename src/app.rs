use eframe::egui::{self, Color32, Stroke, Vec2, Pos2, Rect, Align2, FontId, Rounding, Shadow};
use crate::engine::{Game, map::{MAP_W, MAP_H, Tile, Map}};
use crate::engine::input::handle_input;
use crate::rendering::draw::draw_scene;

const TILE_SIZE: f32 = 32.0;
const MINIMAP_SIZE: f32 = 180.0;
const PANEL_W: f32 = 270.0;
const TOP_BAR_H: f32 = 48.0;
const BOTTOM_PANEL_H: f32 = 120.0;

// Color Palette
fn bg_dark() -> Color32 { Color32::from_rgba_premultiplied(10, 12, 18, 240) }
fn bg_mid() -> Color32 { Color32::from_rgba_premultiplied(18, 22, 32, 220) }
fn bg_panel() -> Color32 { Color32::from_rgba_premultiplied(20, 25, 38, 235) }
fn accent() -> Color32 { Color32::from_rgb(90, 200, 120) }
fn accent_dim() -> Color32 { Color32::from_rgba_premultiplied(90, 200, 120, 60) }
fn text_bright() -> Color32 { Color32::from_rgb(220, 225, 235) }
fn text_dim() -> Color32 { Color32::from_rgb(130, 140, 155) }
fn border_col() -> Color32 { Color32::from_rgba_premultiplied(90, 200, 120, 80) }
fn border_dim() -> Color32 { Color32::from_rgba_premultiplied(60, 75, 100, 120) }
fn warning_col() -> Color32 { Color32::from_rgb(220, 180, 50) }
fn danger_col() -> Color32 { Color32::from_rgb(200, 60, 60) }

pub struct RtsApp {
    pub game: Game,
    last_time: f64,
    fps_history: Vec<f32>,
    frame_count: u32,
    show_help: bool,
    viewport_rect: Rect,
    box_select_world_start: Option<Pos2>,
    hover_world_pos: Option<Pos2>,
    hover_tile: Option<(i32, i32)>,
    log_messages: Vec<String>,
}

impl RtsApp {
    pub fn new(ctx: &egui::Context) -> Self {
        // Set dark visuals
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(text_bright());
        visuals.window_fill = bg_panel();
        visuals.panel_fill = bg_panel();
        ctx.set_visuals(visuals);

        Self {
            game: Game::new(),
            last_time: 0.0,
            fps_history: vec![60.0; 60],
            frame_count: 0,
            show_help: false,
            viewport_rect: Rect::NOTHING,
            box_select_world_start: None,
            hover_world_pos: None,
            hover_tile: None,
            log_messages: vec![
                "Engine initialized.".into(),
                "5 units deployed.".into(),
                "Ready for orders.".into(),
            ],
        }
    }

    fn add_log(&mut self, msg: impl Into<String>) {
        self.log_messages.push(msg.into());
        if self.log_messages.len() > 12 {
            self.log_messages.remove(0);
        }
    }
}

impl eframe::App for RtsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Delta time
        let now = ctx.input(|i| i.time);
        let dt = if self.last_time == 0.0 { 0.016 } else { (now - self.last_time).clamp(0.001, 0.05) as f32 };
        self.last_time = now;
        self.frame_count += 1;

        // FPS tracking
        let fps = if dt > 0.0 { 1.0 / dt } else { 60.0 };
        self.fps_history.push(fps);
        if self.fps_history.len() > 60 { self.fps_history.remove(0); }

        // Update game
        self.game.update(dt);

        // TOP BAR
        egui::TopBottomPanel::top("top_bar")
            .exact_height(TOP_BAR_H)
            .frame(egui::Frame::none().fill(bg_dark()).inner_margin(egui::Margin::symmetric(16.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // Logo/Title
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("⚔")
                            .size(22.0)
                            .color(accent()),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("STRATEGY RTS ENGINE")
                            .size(14.0)
                            .strong()
                            .color(text_bright()),
                    );

                    ui.separator();

                    // Resources mock
                    ui.add_space(8.0);
                    resource_badge(ui, "⚙", "Gold", "1,240");
                    ui.add_space(6.0);
                    resource_badge(ui, "🪵", "Wood", "875");
                    ui.add_space(6.0);
                    resource_badge(ui, "⚒", "Iron", "312");

                    ui.separator();

                    // Clock mock
                    let time_s = self.game.time as u32;
                    let mins = time_s / 60;
                    let secs = time_s % 60;
                    ui.label(
                        egui::RichText::new(format!("🕐 {:02}:{:02}", mins, secs))
                            .size(12.0)
                            .color(text_dim()),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let avg_fps = self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32;
                        let fps_color = if avg_fps > 50.0 { accent() } else if avg_fps > 30.0 { warning_col() } else { danger_col() };
                        ui.label(egui::RichText::new(format!("{:.0} FPS", avg_fps)).size(11.0).color(fps_color));
                        ui.add_space(8.0);

                        if styled_button(ui, "?", text_dim()).clicked() {
                            self.show_help = !self.show_help;
                        }
                        ui.add_space(4.0);

                        let mm_label = if self.game.minimap_visible { "▣" } else { "□" };
                        if styled_button(ui, mm_label, text_dim()).clicked() {
                            self.game.minimap_visible = !self.game.minimap_visible;
                        }
                    });
                });
            });

        // LEFT SIDEBAR
        egui::SidePanel::left("left_panel")
            .exact_width(PANEL_W)
            .frame(egui::Frame::none().fill(bg_panel()).inner_margin(egui::Margin::same(12.0)))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                section_header(ui, "SELECTED UNITS");
                ui.add_space(6.0);

                let selected_units: Vec<_> = self.game.units.iter().filter(|u| u.selected).cloned().collect();

                if selected_units.is_empty() {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("No units selected").color(text_dim()).size(12.0).italics());
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Click a unit to select it,\nor drag to box-select.").color(text_dim()).size(11.0));
                } else {
                    for su in &selected_units {
                        unit_card(ui, su);
                        ui.add_space(4.0);
                    }
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                section_header(ui, "ALL UNITS");
                ui.add_space(6.0);

                egui::ScrollArea::vertical()
                    .id_source("units_scroll")
                    .max_height(240.0)
                    .show(ui, |ui| {
                        let units_snap: Vec<_> = self.game.units.iter().cloned().collect();
                        let mut select_id: Option<usize> = None;
                        let mut focus_id: Option<usize> = None;

                        for u in &units_snap {
                            let is_sel = u.selected;
                            let row_bg = if is_sel { accent_dim() } else { Color32::TRANSPARENT };

                            let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 32.0), egui::Sense::click());
                            painter.rect_filled(resp.rect, 6.0, row_bg);
                            if is_sel {
                                painter.rect_stroke(resp.rect, 6.0, Stroke::new(1.0, border_col()));
                            }

                            // Type icon
                            let icon = match u.unit_type {
                                crate::engine::unit::UnitType::Warrior => "⚔",
                                crate::engine::unit::UnitType::Archer => "🏹",
                                crate::engine::unit::UnitType::Scout => "👁",
                            };
                            painter.text(
                                Pos2::new(resp.rect.min.x + 12.0, resp.rect.center().y),
                                Align2::LEFT_CENTER,
                                icon,
                                FontId::proportional(14.0),
                                text_bright(),
                            );
                            painter.text(
                                Pos2::new(resp.rect.min.x + 32.0, resp.rect.center().y),
                                Align2::LEFT_CENTER,
                                format!("{} #{}", u.unit_type.name(), u.id),
                                FontId::proportional(12.0),
                                if is_sel { text_bright() } else { text_dim() },
                            );

                            // HP bar on the right
                            let hp = (u.health / u.max_health).clamp(0.0, 1.0);
                            let bar_rect = Rect::from_min_size(
                                Pos2::new(resp.rect.max.x - 45.0, resp.rect.center().y - 3.0),
                                Vec2::new(40.0, 6.0),
                            );
                            painter.rect_filled(bar_rect, 3.0, Color32::from_rgba_premultiplied(0, 0, 0, 120));
                            let hp_col = if hp > 0.6 { Color32::from_rgb(50, 200, 70) } else if hp > 0.3 { warning_col() } else { danger_col() };
                            painter.rect_filled(
                                Rect::from_min_size(bar_rect.min, Vec2::new(40.0 * hp, 6.0)),
                                3.0, hp_col,
                            );

                            if resp.clicked() {
                                select_id = Some(u.id);
                            }
                            if resp.double_clicked() {
                                focus_id = Some(u.id);
                            }
                        }

                        // Apply selection clicks
                        if let Some(id) = select_id {
                            if let Some(u) = self.game.units.iter_mut().find(|u| u.id == id) {
                                u.selected = !u.selected;
                            }
                        }
                        if let Some(id) = focus_id {
                            if let Some(u) = self.game.units.iter().find(|u| u.id == id) {
                                let cam_target_x = u.pixel_x * self.game.camera.zoom - 400.0;
                                let cam_target_y = u.pixel_y * self.game.camera.zoom - 300.0;
                                self.game.camera.target_offset = Vec2::new(-cam_target_x + 400.0, -cam_target_y + 300.0);
                            }
                        }
                    });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                section_header(ui, "CAMERA");
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Zoom").size(11.0).color(text_dim()));
                    ui.add_space(4.0);
                    let mut zoom = self.game.camera.target_zoom;
                    let resp = ui.add(
                        egui::Slider::new(&mut zoom, 0.4..=5.0)
                            .show_value(true)
                            .fixed_decimals(1)
                            .suffix("x")
                            .text_color(text_bright()),
                    );
                    if resp.changed() {
                        self.game.camera.target_zoom = zoom;
                    }
                });
                ui.add_space(4.0);

                if ui.button(egui::RichText::new("🏠 Reset Camera").size(11.0).color(text_dim())).clicked() {
                    self.game.camera.target_offset = Vec2::new(50.0, 50.0);
                    self.game.camera.target_zoom = 1.5;
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                section_header(ui, "TERRAIN INFO");
                ui.add_space(6.0);
                if let Some((tx, ty)) = self.hover_tile {
                    if Map::in_bounds(tx, ty) {
                        let tile = self.game.map.tiles[Map::idx(tx, ty)];
                        let tile_name = match tile {
                            Tile::Grass => "Grassland",
                            Tile::DarkGrass => "Dense Grass",
                            Tile::Wall => "Stone Wall",
                            Tile::Water => "Water",
                            Tile::Sand => "Sandy Shore",
                            Tile::Forest => "Forest",
                            Tile::Road => "Road",
                        };
                        let walkable = tile.walkable();
                        let cost = if walkable { format!("{}", tile.move_cost()) } else { "—".into() };

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Tile:").size(11.0).color(text_dim()));
                            ui.label(egui::RichText::new(tile_name).size(11.0).color(text_bright()));
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Position:").size(11.0).color(text_dim()));
                            ui.label(egui::RichText::new(format!("({}, {})", tx, ty)).size(11.0).color(text_bright()));
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Walkable:").size(11.0).color(text_dim()));
                            let wc = if walkable { accent() } else { danger_col() };
                            ui.label(egui::RichText::new(if walkable { "Yes" } else { "No" }).size(11.0).color(wc));
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Move Cost:").size(11.0).color(text_dim()));
                            ui.label(egui::RichText::new(cost).size(11.0).color(text_bright()));
                        });
                    }
                } else {
                    ui.label(egui::RichText::new("Hover over the map").size(11.0).color(text_dim()).italics());
                }
            });

        // BOTTOM PANEL (Event Log + Controls)
        egui::TopBottomPanel::bottom("bottom_panel")
            .exact_height(BOTTOM_PANEL_H)
            .frame(egui::Frame::none().fill(bg_dark()).inner_margin(egui::Margin::symmetric(16.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Event log
                    ui.vertical(|ui| {
                        section_header(ui, "EVENT LOG");
                        ui.add_space(4.0);
                        egui::ScrollArea::vertical()
                            .id_source("log_scroll")
                            .max_height(72.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                let msgs: Vec<_> = self.log_messages.iter().cloned().collect();
                                for (i, msg) in msgs.iter().enumerate() {
                                    let alpha = ((i + 1) as f32 / msgs.len() as f32 * 180.0) as u8 + 75;
                                    let color = Color32::from_rgba_premultiplied(130, 200, 150, alpha);
                                    ui.label(egui::RichText::new(format!("▸ {}", msg)).size(11.0).color(color));
                                }
                            });
                    });

                    ui.add_space(24.0);
                    ui.separator();
                    ui.add_space(24.0);

                    // Control reference
                    ui.vertical(|ui| {
                        section_header(ui, "CONTROLS");
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            control_hint(ui, "Left Click", "Select / Move");
                            ui.add_space(16.0);
                            control_hint(ui, "Right Drag", "Pan Camera");
                        });
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            control_hint(ui, "Scroll", "Zoom");
                            ui.add_space(16.0);
                            control_hint(ui, "A", "Select All");
                        });
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            control_hint(ui, "Esc", "Deselect");
                            ui.add_space(16.0);
                            control_hint(ui, "Double-click list", "Focus Unit");
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Unit summary badges
                        ui.vertical(|ui| {
                            ui.add_space(8.0);
                            section_header(ui, "FORCE SUMMARY");
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                let warriors = self.game.units.iter().filter(|u| matches!(u.unit_type, crate::engine::unit::UnitType::Warrior)).count();
                                let archers = self.game.units.iter().filter(|u| matches!(u.unit_type, crate::engine::unit::UnitType::Archer)).count();
                                let scouts = self.game.units.iter().filter(|u| matches!(u.unit_type, crate::engine::unit::UnitType::Scout)).count();
                                force_badge(ui, "⚔", warriors, Color32::from_rgb(200, 80, 80));
                                ui.add_space(6.0);
                                force_badge(ui, "🏹", archers, Color32::from_rgb(80, 180, 80));
                                ui.add_space(6.0);
                                force_badge(ui, "👁", scouts, Color32::from_rgb(80, 140, 220));
                            });
                        });
                    });
                });
            });

        // MAIN VIEWPORT
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(8, 10, 14)))
            .show(ctx, |ui| {
                let avail = ui.available_rect_before_wrap();
                self.viewport_rect = avail;

                // Allocate the full area
                let (response, painter) = ui.allocate_painter(
                    avail.size(),
                    egui::Sense::click_and_drag(),
                );

                let viewport_origin = avail.min;

                // Handle game input
                let input = ui.input(|i| i.clone());
                handle_input(&mut self.game, &input, viewport_origin);

                // Update hover tile
                self.hover_tile = input.pointer.hover_pos().map(|p| {
                    let local = Pos2::new(p.x - viewport_origin.x, p.y - viewport_origin.y);
                    let world = self.game.camera.screen_to_world(local);
                    ((world.x / TILE_SIZE) as i32, (world.y / TILE_SIZE) as i32)
                });

                // Draw the scene
                draw_scene(&self.game, response.rect, painter);

                // Box select overlay
                if let Some(start) = self.game.input_handler.box_select_start {
                    if let Some(hover) = input.pointer.hover_pos() {
                        let local_hover = Pos2::new(hover.x - viewport_origin.x, hover.y - viewport_origin.y);
                        let world_hover = self.game.camera.screen_to_world(local_hover);
                        let s_screen = self.game.camera.world_to_screen(start);
                        let e_screen = self.game.camera.world_to_screen(world_hover);
                        let sel_rect = Rect::from_two_pos(
                            Pos2::new(s_screen.x + viewport_origin.x, s_screen.y + viewport_origin.y),
                            Pos2::new(e_screen.x + viewport_origin.x, e_screen.y + viewport_origin.y),
                        );
                        ui.painter().rect_stroke(sel_rect, 2.0, Stroke::new(1.5, Color32::from_rgba_premultiplied(100, 220, 100, 200)));
                        ui.painter().rect_filled(sel_rect, 2.0, Color32::from_rgba_premultiplied(60, 150, 60, 25));
                    }
                }

                // Hover tile highlight
                if let Some((tx, ty)) = self.hover_tile {
                    if Map::in_bounds(tx, ty) {
                        let tile = self.game.map.tiles[Map::idx(tx, ty)];
                        let highlight_color = if tile.walkable() {
                            Color32::from_rgba_premultiplied(100, 200, 100, 35)
                        } else {
                            Color32::from_rgba_premultiplied(200, 60, 60, 35)
                        };
                        let border_color = if tile.walkable() {
                            Color32::from_rgba_premultiplied(100, 200, 100, 120)
                        } else {
                            Color32::from_rgba_premultiplied(200, 60, 60, 120)
                        };

                        let world_center = Pos2::new(tx as f32 * TILE_SIZE + TILE_SIZE / 2.0, ty as f32 * TILE_SIZE + TILE_SIZE / 2.0);
                        let screen_center = self.game.camera.world_to_screen(world_center);
                        let half = TILE_SIZE * self.game.camera.zoom / 2.0;
                        let tile_screen_rect = Rect::from_center_size(
                            Pos2::new(screen_center.x + viewport_origin.x, screen_center.y + viewport_origin.y),
                            Vec2::splat(TILE_SIZE * self.game.camera.zoom),
                        );
                        ui.painter().rect_filled(tile_screen_rect, 0.0, highlight_color);
                        ui.painter().rect_stroke(tile_screen_rect, 0.0, Stroke::new(1.5, border_color));
                    }
                }

                // MINIMAP
                if self.game.minimap_visible {
                    let map_rect = Rect::from_min_size(
                        Pos2::new(avail.max.x - MINIMAP_SIZE - 12.0, avail.max.y - MINIMAP_SIZE - 12.0),
                        Vec2::splat(MINIMAP_SIZE),
                    );

                    let mm_painter = ui.painter();

                    // Background
                    mm_painter.rect_filled(map_rect, 6.0, Color32::from_rgba_premultiplied(8, 10, 16, 220));
                    mm_painter.rect_stroke(map_rect, 6.0, Stroke::new(1.5, border_col()));

                    // Draw terrain
                    let scale_x = MINIMAP_SIZE / (MAP_W as f32 * TILE_SIZE);
                    let scale_y = MINIMAP_SIZE / (MAP_H as f32 * TILE_SIZE);

                    for y in 0..MAP_H {
                        for x in 0..MAP_W {
                            let tile = self.game.map.tiles[Map::idx(x, y)];
                            let tile_color = match tile {
                                Tile::Grass => Color32::from_rgb(40, 85, 35),
                                Tile::DarkGrass => Color32::from_rgb(28, 65, 25),
                                Tile::Wall => Color32::from_rgb(80, 75, 70),
                                Tile::Water => Color32::from_rgb(30, 80, 160),
                                Tile::Sand => Color32::from_rgb(180, 160, 100),
                                Tile::Forest => Color32::from_rgb(22, 55, 18),
                                Tile::Road => Color32::from_rgb(110, 95, 72),
                            };
                            let px = map_rect.min.x + x as f32 * TILE_SIZE * scale_x;
                            let py = map_rect.min.y + y as f32 * TILE_SIZE * scale_y;
                            let pw = (TILE_SIZE * scale_x).ceil();
                            let ph = (TILE_SIZE * scale_y).ceil();
                            mm_painter.rect_filled(
                                Rect::from_min_size(Pos2::new(px, py), Vec2::new(pw, ph)),
                                0.0, tile_color,
                            );
                        }
                    }

                    // Draw units on minimap
                    for u in &self.game.units {
                        let ux = map_rect.min.x + u.pixel_x * scale_x;
                        let uy = map_rect.min.y + u.pixel_y * scale_y;
                        let unit_color = match u.unit_type {
                            crate::engine::unit::UnitType::Warrior => Color32::from_rgb(220, 80, 80),
                            crate::engine::unit::UnitType::Archer => Color32::from_rgb(80, 200, 80),
                            crate::engine::unit::UnitType::Scout => Color32::from_rgb(80, 140, 240),
                        };
                        mm_painter.circle_filled(Pos2::new(ux, uy), if u.selected { 4.0 } else { 2.5 }, unit_color);
                        if u.selected {
                            mm_painter.circle_stroke(Pos2::new(ux, uy), 5.0, Stroke::new(1.0, Color32::from_rgb(255, 220, 50)));
                        }
                    }

                    // Viewport indicator on minimap
                    let vp_world_min = self.game.camera.screen_to_world(Pos2::ZERO);
                    let vp_world_max = self.game.camera.screen_to_world(Pos2::new(avail.width(), avail.height()));
                    let vp_mm_min = Pos2::new(
                        map_rect.min.x + vp_world_min.x * scale_x,
                        map_rect.min.y + vp_world_min.y * scale_y,
                    );
                    let vp_mm_max = Pos2::new(
                        map_rect.min.x + vp_world_max.x * scale_x,
                        map_rect.min.y + vp_world_max.y * scale_y,
                    );
                    let vp_rect = Rect::from_two_pos(vp_mm_min, vp_mm_max);
                    mm_painter.rect_stroke(vp_rect, 0.0, Stroke::new(1.5, Color32::from_rgba_premultiplied(255, 255, 255, 180)));

                    // Minimap title
                    mm_painter.text(
                        Pos2::new(map_rect.min.x + MINIMAP_SIZE / 2.0, map_rect.min.y + 8.0),
                        Align2::CENTER_TOP,
                        "MINIMAP",
                        FontId::proportional(9.0),
                        text_dim(),
                    );
                }

                // Zoom indicator bottom right
                {
                    let zoom_pos = Pos2::new(
                        avail.max.x - (if self.game.minimap_visible { MINIMAP_SIZE + 24.0 } else { 16.0 }),
                        avail.max.y - 24.0,
                    );
                    ui.painter().text(
                        zoom_pos,
                        Align2::RIGHT_BOTTOM,
                        format!("⊕ {:.1}x", self.game.camera.zoom),
                        FontId::proportional(11.0),
                        text_dim(),
                    );
                }

                // Selection count badge
                let sel_count = self.game.selected_count();
                if sel_count > 0 {
                    let badge_pos = Pos2::new(avail.min.x + 12.0, avail.max.y - 12.0);
                    ui.painter().text(
                        badge_pos,
                        Align2::LEFT_BOTTOM,
                        format!("✔ {} unit{} selected", sel_count, if sel_count == 1 { "" } else { "s" }),
                        FontId::proportional(12.0),
                        accent(),
                    );
                }

                // Coordinate display
                if let Some((tx, ty)) = self.hover_tile {
                    let coord_pos = Pos2::new(avail.min.x + 12.0, avail.min.y + 12.0);
                    ui.painter().text(
                        coord_pos,
                        Align2::LEFT_TOP,
                        format!("({}, {})", tx, ty),
                        FontId::proportional(11.0),
                        text_dim(),
                    );
                }
            });

        // HELP MODAL
        if self.show_help {
            egui::Window::new("Help — Strategy RTS Engine")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(egui::Frame::window(&ctx.style()).fill(bg_panel()).stroke(Stroke::new(1.5, border_col())))
                .show(ctx, |ui| {
                    ui.set_min_width(360.0);
                    ui.add_space(4.0);

                    section_header(ui, "CONTROLS");
                    ui.add_space(8.0);

                    help_row(ui, "Left Click on unit", "Select / deselect unit");
                    help_row(ui, "Left Click on ground", "Move selected units");
                    help_row(ui, "Right Mouse Drag", "Pan the camera");
                    help_row(ui, "Mouse Scroll", "Zoom in / out");
                    help_row(ui, "[A]", "Select all units");
                    help_row(ui, "[Esc]", "Deselect all units");
                    help_row(ui, "Double-click unit row", "Focus camera on unit");

                    ui.add_space(12.0);
                    section_header(ui, "UNIT TYPES");
                    ui.add_space(8.0);

                    help_row(ui, "⚔ Warrior", "Slow, heavy — 120 HP");
                    help_row(ui, "🏹 Archer", "Medium, agile — 70 HP");
                    help_row(ui, "👁 Scout", "Fast, light — 90 HP");

                    ui.add_space(12.0);
                    section_header(ui, "TERRAIN");
                    ui.add_space(8.0);

                    help_row(ui, "Road", "Fastest movement (cost 1)");
                    help_row(ui, "Grass", "Normal movement (cost 2)");
                    help_row(ui, "Sand", "Slow movement (cost 3)");
                    help_row(ui, "Wall / Water / Forest", "Impassable");

                    ui.add_space(12.0);
                    if ui.button(egui::RichText::new("Close").color(accent())).clicked() {
                        self.show_help = false;
                    }
                });
        }

        ctx.request_repaint();
    }
}

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).size(10.0).color(Color32::from_rgb(90, 200, 120)).strong());
        ui.add_space(4.0);
        ui.add(egui::Separator::default().horizontal().spacing(4.0));
    });
}

fn unit_card(ui: &mut egui::Ui, unit: &crate::engine::unit::Unit) {
    let frame = egui::Frame::none()
        .fill(Color32::from_rgba_premultiplied(30, 40, 55, 200))
        .stroke(Stroke::new(1.0, border_col()))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(10.0, 6.0));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        let icon = match unit.unit_type {
            crate::engine::unit::UnitType::Warrior => "⚔",
            crate::engine::unit::UnitType::Archer => "🏹",
            crate::engine::unit::UnitType::Scout => "👁",
        };
        let unit_color = match unit.unit_type {
            crate::engine::unit::UnitType::Warrior => Color32::from_rgb(220, 100, 100),
            crate::engine::unit::UnitType::Archer => Color32::from_rgb(100, 200, 100),
            crate::engine::unit::UnitType::Scout => Color32::from_rgb(100, 160, 240),
        };

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(icon).size(18.0).color(unit_color));
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(format!("{} #{}", unit.unit_type.name(), unit.id)).size(12.0).color(text_bright()).strong());
                ui.label(egui::RichText::new(format!("Pos ({}, {})  {}", unit.x, unit.y, if unit.is_moving { "→ Moving" } else { "○ Idle" }))
                    .size(10.0).color(if unit.is_moving { accent() } else { text_dim() }));
            });
        });

        ui.add_space(4.0);

        // HP bar
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("HP").size(10.0).color(text_dim()));
            ui.add_space(4.0);
            let hp_ratio = (unit.health / unit.max_health).clamp(0.0, 1.0);
            let hp_col = if hp_ratio > 0.6 { Color32::from_rgb(50, 200, 70) } else if hp_ratio > 0.3 { warning_col() } else { danger_col() };

            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width() - 40.0, 8.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 4.0, Color32::from_rgba_premultiplied(0, 0, 0, 120));
            ui.painter().rect_filled(
                Rect::from_min_size(rect.min, Vec2::new(rect.width() * hp_ratio, rect.height())),
                4.0, hp_col,
            );
            ui.label(egui::RichText::new(format!("{}/{}", unit.health as i32, unit.max_health as i32)).size(10.0).color(text_dim()));
        });
    });
}

fn resource_badge(ui: &mut egui::Ui, icon: &str, name: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).size(13.0));
        ui.label(egui::RichText::new(value).size(12.0).strong().color(Color32::from_rgb(220, 200, 140)));
        ui.label(egui::RichText::new(name).size(10.0).color(text_dim()));
    });
}

fn force_badge(ui: &mut egui::Ui, icon: &str, count: usize, color: Color32) {
    let frame = egui::Frame::none()
        .fill(Color32::from_rgba_premultiplied(20, 25, 38, 200))
        .stroke(Stroke::new(1.0, border_dim()))
        .rounding(Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(8.0, 4.0));
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(icon).size(14.0).color(color));
            ui.label(egui::RichText::new(format!("×{}", count)).size(12.0).color(text_bright()));
        });
    });
}

fn control_hint(ui: &mut egui::Ui, key: &str, desc: &str) {
    ui.horizontal(|ui| {
        // Key pill
        let frame = egui::Frame::none()
            .fill(Color32::from_rgba_premultiplied(40, 50, 70, 200))
            .stroke(Stroke::new(1.0, border_dim()))
            .rounding(Rounding::same(3.0))
            .inner_margin(egui::Margin::symmetric(5.0, 2.0));
        frame.show(ui, |ui| {
            ui.label(egui::RichText::new(key).size(10.0).color(text_bright()).monospace());
        });
        ui.add_space(3.0);
        ui.label(egui::RichText::new(desc).size(10.0).color(text_dim()));
    });
}

fn help_row(ui: &mut egui::Ui, key: &str, desc: &str) {
    ui.horizontal(|ui| {
        let frame = egui::Frame::none()
            .fill(Color32::from_rgba_premultiplied(40, 50, 70, 200))
            .stroke(Stroke::new(1.0, border_dim()))
            .rounding(Rounding::same(3.0))
            .inner_margin(egui::Margin::symmetric(6.0, 3.0));
        frame.show(ui, |ui| {
            ui.set_min_width(130.0);
            ui.label(egui::RichText::new(key).size(11.0).color(accent()).monospace());
        });
        ui.add_space(8.0);
        ui.label(egui::RichText::new(desc).size(11.0).color(text_bright()));
    });
    ui.add_space(3.0);
}

fn styled_button(ui: &mut egui::Ui, label: &str, color: Color32) -> egui::Response {
    let btn = egui::Button::new(egui::RichText::new(label).size(12.0).color(color))
        .fill(Color32::from_rgba_premultiplied(30, 38, 55, 200))
        .stroke(Stroke::new(1.0, border_dim()))
        .rounding(Rounding::same(4.0));
    ui.add(btn)
}