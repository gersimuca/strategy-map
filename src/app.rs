use eframe::egui::{self, Color32, FontId, Align2, Pos2, Rect, Rounding, Stroke, Vec2};
use crate::engine::{
    Game,
    map::{MAP_W, MAP_H, Tile, Map},
    unit::UnitType,
    enemy::EnemyType,
};
use crate::engine::input::handle_input;
use crate::rendering::draw::draw_scene;

const TILE_SIZE:    f32 = 32.0;
const MINIMAP_SIZE: f32 = 180.0;
const PANEL_W:      f32 = 268.0;
const TOP_BAR_H:    f32 = 46.0;
const BOTTOM_H:     f32 = 116.0;

// ── Colour palette ─────────────────────────────────────────────────
fn bg0()      -> Color32 { Color32::from_rgba_premultiplied(8,  10, 16, 248) }
fn bg1()      -> Color32 { Color32::from_rgba_premultiplied(14, 18, 28, 235) }
fn bg2()      -> Color32 { Color32::from_rgba_premultiplied(20, 26, 40, 228) }
fn acc()      -> Color32 { Color32::from_rgb(80, 195, 110) }
fn acc_dim()  -> Color32 { Color32::from_rgba_premultiplied(80, 195, 110, 55) }
fn t1()       -> Color32 { Color32::from_rgb(218, 224, 234) }
fn t2()       -> Color32 { Color32::from_rgb(125, 136, 155) }
fn bord()     -> Color32 { Color32::from_rgba_premultiplied(80, 195, 110, 75) }
fn bord2()    -> Color32 { Color32::from_rgba_premultiplied(55, 72, 100, 110) }
fn warn()     -> Color32 { Color32::from_rgb(218, 178, 45) }
fn dng()      -> Color32 { Color32::from_rgb(195, 55, 55) }
fn unit_col(t: &UnitType) -> Color32 {
    match t {
        UnitType::Warrior => Color32::from_rgb(215, 85, 85),
        UnitType::Archer  => Color32::from_rgb(85, 195, 85),
        UnitType::Scout   => Color32::from_rgb(85, 135, 235),
    }
}
fn enemy_col(t: &EnemyType) -> Color32 {
    match t {
        EnemyType::Grunt  => Color32::from_rgb(195, 55, 55),
        EnemyType::Brute  => Color32::from_rgb(150, 25, 25),
        EnemyType::Archer => Color32::from_rgb(185, 85, 30),
    }
}

// App state
pub struct RtsApp {
    pub game:       Game,
    last_time:      f64,
    fps_history:    Vec<f32>,
    show_help:      bool,
    hover_tile:     Option<(i32, i32)>,
    log_messages:   Vec<(String, bool)>,   // (text, is_combat)
    viewport_rect:  Rect,
}

impl RtsApp {
    pub fn new(ctx: &egui::Context) -> Self {
        let mut vis = egui::Visuals::dark();
        vis.override_text_color = Some(t1());
        vis.window_fill         = bg2();
        vis.panel_fill          = bg1();
        ctx.set_visuals(vis);

        Self {
            game:          Game::new(),
            last_time:     0.0,
            fps_history:   vec![60.0; 60],
            show_help:     false,
            hover_tile:    None,
            log_messages:  vec![
                ("RTS Engine v1.0 started.".into(), false),
                ("5 units deployed.".into(), false),
                ("Enemy forces nearby!".into(), true),
                ("Wave 1 incoming in 4 s.".into(), true),
            ],
            viewport_rect: Rect::NOTHING,
        }
    }

    fn push_log(&mut self, msg: impl Into<String>, combat: bool) {
        self.log_messages.push((msg.into(), combat));
        if self.log_messages.len() > 16 { self.log_messages.remove(0); }
    }
}


fn sec(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).size(9.5).color(acc()).strong());
        ui.add(egui::Separator::default().horizontal().spacing(4.0));
    });
}

fn key_hint(ui: &mut egui::Ui, key: &str, desc: &str) {
    ui.horizontal(|ui| {
        egui::Frame::none()
            .fill(Color32::from_rgba_premultiplied(35, 45, 65, 200))
            .stroke(Stroke::new(1.0, bord2()))
            .rounding(Rounding::same(3.0))
            .inner_margin(egui::Margin::symmetric(5.0, 2.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(key).size(10.0).color(t1()).monospace());
            });
        ui.add_space(3.0);
        ui.label(egui::RichText::new(desc).size(10.0).color(t2()));
    });
}

fn hp_bar_row(ui: &mut egui::Ui, hp: f32, max_hp: f32, label: bool) {
    ui.horizontal(|ui| {
        if label { ui.label(egui::RichText::new("HP").size(9.5).color(t2())); ui.add_space(3.0); }
        let ratio = (hp / max_hp).clamp(0.0, 1.0);
        let col   = if ratio > 0.6 { Color32::from_rgb(45, 195, 60) }
        else if ratio > 0.3 { warn() } else { dng() };
        let w = ui.available_width() - if label { 42.0 } else { 0.0 };
        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 7.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 3.5, Color32::from_rgba_premultiplied(0, 0, 0, 130));
        if ratio > 0.0 {
            ui.painter().rect_filled(
                Rect::from_min_size(rect.min, Vec2::new(rect.width() * ratio, rect.height())),
                3.5, col,
            );
        }
        if label {
            ui.label(egui::RichText::new(format!("{}/{}", hp as i32, max_hp as i32)).size(9.0).color(t2()));
        }
    });
}

fn stat_pill(ui: &mut egui::Ui, icon: &str, count: usize, col: Color32) {
    egui::Frame::none()
        .fill(Color32::from_rgba_premultiplied(18, 22, 35, 200))
        .stroke(Stroke::new(1.0, bord2()))
        .rounding(Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(7.0, 3.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(13.0).color(col));
                ui.label(egui::RichText::new(format!("×{}", count)).size(11.0).color(t1()));
            });
        });
}

fn res_badge(ui: &mut egui::Ui, icon: &str, name: &str, val: u32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).size(12.0));
        ui.label(egui::RichText::new(val.to_string()).size(11.5).strong().color(Color32::from_rgb(218, 198, 136)));
        ui.label(egui::RichText::new(name).size(9.5).color(t2()));
    });
}

fn icon_btn(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).size(12.0).color(t2()))
            .fill(Color32::from_rgba_premultiplied(28, 36, 52, 200))
            .stroke(Stroke::new(1.0, bord2()))
            .rounding(Rounding::same(4.0)),
    )
}

impl eframe::App for RtsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Delta time
        let now = ctx.input(|i| i.time);
        let dt  = if self.last_time == 0.0 { 0.016 }
        else { (now - self.last_time).clamp(0.001, 0.05) as f32 };
        self.last_time = now;

        let fps = 1.0 / dt;
        self.fps_history.push(fps);
        if self.fps_history.len() > 60 { self.fps_history.remove(0); }

        // Track kills / losses before update
        let pre_killed = self.game.combat.killed_enemies;
        let pre_lost   = self.game.combat.lost_units;
        let pre_wave   = self.game.wave_number;

        self.game.update(dt);

        // Auto-log events
        if self.game.combat.killed_enemies > pre_killed {
            let n = self.game.combat.killed_enemies - pre_killed;
            self.push_log(format!("{} enem{} slain!", n, if n == 1 { "y" } else { "ies" }), true);
        }
        if self.game.combat.lost_units > pre_lost {
            let n = self.game.combat.lost_units - pre_lost;
            self.push_log(format!("{} unit{} lost!", n, if n == 1 { "" } else { "s" }), true);
        }
        if self.game.wave_number > pre_wave {
            self.push_log(format!("⚠ Wave {} spawned!", self.game.wave_number), true);
        }

        // TOP BAR
        egui::TopBottomPanel::top("top_bar")
            .exact_height(TOP_BAR_H)
            .frame(egui::Frame::none().fill(bg0()).inner_margin(egui::Margin::symmetric(14.0, 7.0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(egui::RichText::new("⚔").size(21.0).color(acc()));
                    ui.add_space(3.0);
                    ui.label(egui::RichText::new("STRATEGY RTS").size(13.0).strong().color(t1()));
                    ui.separator();

                    // Resources
                    let gold = 1240 + (self.game.time * 1.5) as u32;
                    let iron = 312  + self.game.combat.killed_enemies * 5;
                    res_badge(ui, "◈", "Gold", gold);
                    ui.add_space(4.0);
                    res_badge(ui, "♦", "Iron", iron);
                    ui.separator();

                    // Game clock
                    let ts = self.game.time as u32;
                    ui.label(egui::RichText::new(format!("⏱ {:02}:{:02}", ts / 60, ts % 60)).size(11.0).color(t2()));
                    ui.separator();

                    // Wave info
                    let wc = self.game.wave_countdown as u32;
                    ui.label(
                        egui::RichText::new(format!("Wave {}  ·  Next: {:02}:{:02}", self.game.wave_number, wc / 60, wc % 60))
                            .size(11.0).color(dng()),
                    );
                    ui.separator();

                    // Score
                    ui.label(egui::RichText::new(format!("Score: {}", self.game.combat.score)).size(11.5).color(warn()).strong());

                    // Right-side buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let avg = self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32;
                        let fc  = if avg > 50.0 { acc() } else if avg > 30.0 { warn() } else { dng() };
                        ui.label(egui::RichText::new(format!("{:.0} fps", avg)).size(10.0).color(fc));
                        ui.add_space(6.0);

                        if icon_btn(ui, "?").clicked() { self.show_help = !self.show_help; }
                        ui.add_space(3.0);
                        let mm = if self.game.minimap_visible { "▣" } else { "□" };
                        if icon_btn(ui, mm).clicked() { self.game.minimap_visible = !self.game.minimap_visible; }
                        ui.add_space(3.0);
                        let pp = if self.game.paused { "▶" } else { "⏸" };
                        if icon_btn(ui, pp).clicked() { self.game.paused = !self.game.paused; }
                    });
                });
            });

        // LEFT SIDEBAR
        egui::SidePanel::left("left_panel")
            .exact_width(PANEL_W)
            .frame(egui::Frame::none().fill(bg1()).inner_margin(egui::Margin::same(11.0)))
            .show(ctx, |ui| {
                // Selected units
                sec(ui, "SELECTED UNITS");
                ui.add_space(5.0);

                let sel_snap: Vec<_> = self.game.units.iter().filter(|u| u.selected).cloned().collect();
                if sel_snap.is_empty() {
                    ui.label(egui::RichText::new("No units selected.").color(t2()).size(11.0).italics());
                    ui.label(egui::RichText::new("Click or drag to select.").color(t2()).size(10.0));
                } else {
                    for su in &sel_snap {
                        // Unit card
                        egui::Frame::none()
                            .fill(Color32::from_rgba_premultiplied(26, 36, 52, 210))
                            .stroke(Stroke::new(1.0, bord()))
                            .rounding(Rounding::same(5.0))
                            .inner_margin(egui::Margin::symmetric(9.0, 6.0))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                let icon = unit_icon(&su.unit_type);
                                let col  = unit_col(&su.unit_type);
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(icon).size(17.0).color(col));
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(format!("{} #{}", su.unit_type.name(), su.id))
                                            .size(11.5).color(t1()).strong());
                                        let state_txt = if su.attacking { "⚔ Attacking" }
                                        else if su.is_moving { "→ Moving" } else { "○ Idle" };
                                        let scol = if su.attacking { dng() }
                                        else if su.is_moving { acc() } else { t2() };
                                        ui.label(egui::RichText::new(format!("({},{})  {}", su.x, su.y, state_txt))
                                            .size(9.5).color(scol));
                                    });
                                });
                                ui.add_space(3.0);
                                hp_bar_row(ui, su.health, su.max_health, true);
                            });
                        ui.add_space(3.0);
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // All units list
                sec(ui, "ALL UNITS");
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_source("ul")
                    .max_height(140.0)
                    .show(ui, |ui| {
                        let snap: Vec<_> = self.game.units.iter().cloned().collect();
                        let mut sel_id:   Option<usize> = None;
                        let mut focus_id: Option<usize> = None;

                        for u in &snap {
                            let bg = if u.selected { acc_dim() } else { Color32::TRANSPARENT };
                            let (resp, paint) = ui.allocate_painter(
                                Vec2::new(ui.available_width(), 30.0), egui::Sense::click(),
                            );
                            paint.rect_filled(resp.rect, 5.0, bg);
                            if u.selected { paint.rect_stroke(resp.rect, 5.0, Stroke::new(1.0, bord())); }

                            let icon = unit_icon(&u.unit_type);
                            let col  = unit_col(&u.unit_type);
                            paint.text(Pos2::new(resp.rect.min.x + 9.0,  resp.rect.center().y), Align2::LEFT_CENTER, icon, FontId::proportional(13.0), col);
                            paint.text(Pos2::new(resp.rect.min.x + 27.0, resp.rect.center().y), Align2::LEFT_CENTER,
                                       format!("{} #{}", u.unit_type.name(), u.id),
                                       FontId::proportional(11.0),
                                       if u.selected { t1() } else { t2() },
                            );
                            // mini HP bar
                            let hp  = (u.health / u.max_health).clamp(0.0, 1.0);
                            let bar = Rect::from_min_size(Pos2::new(resp.rect.max.x - 44.0, resp.rect.center().y - 3.0), Vec2::new(38.0, 6.0));
                            paint.rect_filled(bar, 3.0, Color32::from_rgba_premultiplied(0, 0, 0, 120));
                            let hc  = if hp > 0.6 { Color32::from_rgb(45, 195, 60) } else if hp > 0.3 { warn() } else { dng() };
                            if hp > 0.0 { paint.rect_filled(Rect::from_min_size(bar.min, Vec2::new(38.0 * hp, 6.0)), 3.0, hc); }

                            if resp.clicked()        { sel_id   = Some(u.id); }
                            if resp.double_clicked() { focus_id = Some(u.id); }
                        }
                        if let Some(id) = sel_id {
                            if let Some(u) = self.game.units.iter_mut().find(|u| u.id == id) {
                                u.selected = !u.selected;
                            }
                        }
                        if let Some(id) = focus_id {
                            if let Some(u) = self.game.units.iter().find(|u| u.id == id) {
                                let z  = self.game.camera.target_zoom;
                                let px = u.pixel_x; let py = u.pixel_y;
                                self.game.camera.target_offset = egui::Vec2::new(
                                    self.viewport_rect.width()  / 2.0 - px * z,
                                    self.viewport_rect.height() / 2.0 - py * z,
                                );
                            }
                        }
                    });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // Enemy list
                sec(ui, "ENEMIES");
                ui.add_space(4.0);
                let alive_e = self.game.enemies.iter().filter(|e| !e.dead).count();
                ui.label(egui::RichText::new(format!("{} active  ·  {} eliminated", alive_e, self.game.combat.killed_enemies))
                    .size(10.0).color(t2()));
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_source("el")
                    .max_height(115.0)
                    .show(ui, |ui| {
                        let snap: Vec<_> = self.game.enemies.iter().filter(|e| !e.dead).cloned().collect();
                        let mut focus_id: Option<(f32, f32)> = None;
                        for e in &snap {
                            let (resp, paint) = ui.allocate_painter(
                                Vec2::new(ui.available_width(), 26.0), egui::Sense::click(),
                            );
                            paint.rect_filled(resp.rect, 4.0, Color32::from_rgba_premultiplied(28, 8, 8, 160));
                            let icon = enemy_icon(&e.enemy_type);
                            let ecol = enemy_col(&e.enemy_type);
                            paint.text(Pos2::new(resp.rect.min.x + 7.0, resp.rect.center().y), Align2::LEFT_CENTER, icon, FontId::proportional(12.0), ecol);
                            let ai_lbl = match &e.ai_state {
                                crate::engine::enemy::AiState::Patrol           => "Patrol",
                                crate::engine::enemy::AiState::Chase { .. }     => "Chase!",
                                crate::engine::enemy::AiState::Attack { .. }    => "ATTACK",
                            };
                            paint.text(Pos2::new(resp.rect.min.x + 22.0, resp.rect.center().y), Align2::LEFT_CENTER,
                                       format!("{} #{} — {}", e.enemy_type.name(), e.id, ai_lbl),
                                       FontId::proportional(9.5), t2());
                            let hp  = (e.health / e.max_health).clamp(0.0, 1.0);
                            let bar = Rect::from_min_size(Pos2::new(resp.rect.max.x - 40.0, resp.rect.center().y - 3.0), Vec2::new(35.0, 5.0));
                            paint.rect_filled(bar, 3.0, Color32::from_rgba_premultiplied(0, 0, 0, 120));
                            if hp > 0.0 { paint.rect_filled(Rect::from_min_size(bar.min, Vec2::new(35.0 * hp, 5.0)), 3.0, dng()); }
                            if resp.clicked() { focus_id = Some((e.pixel_x, e.pixel_y)); }
                        }
                        if let Some((ex, ey)) = focus_id {
                            let z = self.game.camera.target_zoom;
                            self.game.camera.target_offset = egui::Vec2::new(
                                self.viewport_rect.width()  / 2.0 - ex * z,
                                self.viewport_rect.height() / 2.0 - ey * z,
                            );
                        }
                    });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // Camera zoom slider
                sec(ui, "CAMERA");
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Zoom").size(10.0).color(t2()));
                    let mut z = self.game.camera.target_zoom;
                    if ui.add(egui::Slider::new(&mut z, 0.35..=5.0).fixed_decimals(1).suffix("×")).changed() {
                        self.game.camera.target_zoom = z;
                    }
                });
                ui.add_space(3.0);
                if ui.button(egui::RichText::new("⌂ Reset").size(10.0).color(t2())).clicked() {
                    self.game.camera.target_offset = egui::Vec2::new(80.0, 80.0);
                    self.game.camera.target_zoom   = 1.5;
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // Terrain info
                sec(ui, "TERRAIN");
                ui.add_space(3.0);
                if let Some((tx, ty)) = self.hover_tile {
                    if Map::in_bounds(tx, ty) {
                        let tile = self.game.map.tiles[Map::idx(tx, ty)];
                        let (name, walk, cost) = match tile {
                            Tile::Grass     => ("Grassland",   true,  "2"),
                            Tile::DarkGrass => ("Dense Grass", true,  "2"),
                            Tile::Wall      => ("Stone Wall",  false, "—"),
                            Tile::Water     => ("Water",       false, "—"),
                            Tile::Sand      => ("Sand",        true,  "3"),
                            Tile::Forest    => ("Forest",      false, "—"),
                            Tile::Road      => ("Road",        true,  "1"),
                        };
                        let wc = if walk { acc() } else { dng() };
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Tile:").size(10.0).color(t2()));
                            ui.label(egui::RichText::new(name).size(10.0).color(t1()));
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Pos:").size(10.0).color(t2()));
                            ui.label(egui::RichText::new(format!("({},{})", tx, ty)).size(10.0).color(t1()));
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Walk:").size(10.0).color(t2()));
                            ui.label(egui::RichText::new(if walk { "Yes" } else { "No" }).size(10.0).color(wc));
                            ui.add_space(6.0);
                            ui.label(egui::RichText::new("Cost:").size(10.0).color(t2()));
                            ui.label(egui::RichText::new(cost).size(10.0).color(t1()));
                        });
                    }
                } else {
                    ui.label(egui::RichText::new("Hover over the map.").size(10.0).color(t2()).italics());
                }
            });

        // BOTTOM BAR
        egui::TopBottomPanel::bottom("bottom_bar")
            .exact_height(BOTTOM_H)
            .frame(egui::Frame::none().fill(bg0()).inner_margin(egui::Margin::symmetric(14.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Event log
                    ui.vertical(|ui| {
                        ui.set_width(230.0);
                        sec(ui, "EVENT LOG");
                        ui.add_space(3.0);
                        egui::ScrollArea::vertical()
                            .id_source("log")
                            .max_height(78.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for (msg, is_combat) in &self.log_messages {
                                    let col = if *is_combat {
                                        Color32::from_rgba_premultiplied(200, 120, 100, 220)
                                    } else {
                                        Color32::from_rgba_premultiplied(120, 190, 140, 200)
                                    };
                                    ui.label(egui::RichText::new(format!("▸ {}", msg)).size(10.0).color(col));
                                }
                            });
                    });

                    ui.add_space(18.0);
                    ui.separator();
                    ui.add_space(18.0);

                    // Controls reference
                    ui.vertical(|ui| {
                        ui.set_width(248.0);
                        sec(ui, "CONTROLS");
                        ui.add_space(3.0);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                key_hint(ui, "Left Click",  "Select / Move");
                                key_hint(ui, "Right Drag",  "Pan camera");
                                key_hint(ui, "Scroll",      "Zoom");
                            });
                            ui.add_space(10.0);
                            ui.vertical(|ui| {
                                key_hint(ui, "A",   "Select all");
                                key_hint(ui, "S",   "Stop selected");
                                key_hint(ui, "Esc", "Deselect");
                            });
                        });
                    });

                    // Battle stats (right-aligned)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.vertical(|ui| {
                            sec(ui, "BATTLE STATS");
                            ui.add_space(3.0);
                            // Friendly
                            ui.horizontal(|ui| {
                                let w = self.game.units.iter().filter(|u| matches!(u.unit_type, UnitType::Warrior)).count();
                                let a = self.game.units.iter().filter(|u| matches!(u.unit_type, UnitType::Archer)).count();
                                let s = self.game.units.iter().filter(|u| matches!(u.unit_type, UnitType::Scout)).count();
                                stat_pill(ui, "⚔", w, unit_col(&UnitType::Warrior));
                                ui.add_space(4.0);
                                stat_pill(ui, "🏹", a, unit_col(&UnitType::Archer));
                                ui.add_space(4.0);
                                stat_pill(ui, "◈", s, unit_col(&UnitType::Scout));
                            });
                            ui.add_space(3.0);
                            // Enemy
                            ui.horizontal(|ui| {
                                let eg = self.game.enemies.iter().filter(|e| matches!(e.enemy_type, EnemyType::Grunt)  && !e.dead).count();
                                let eb = self.game.enemies.iter().filter(|e| matches!(e.enemy_type, EnemyType::Brute)  && !e.dead).count();
                                let ea = self.game.enemies.iter().filter(|e| matches!(e.enemy_type, EnemyType::Archer) && !e.dead).count();
                                stat_pill(ui, "✕", eg, enemy_col(&EnemyType::Grunt));
                                ui.add_space(4.0);
                                stat_pill(ui, "◉", eb, enemy_col(&EnemyType::Brute));
                                ui.add_space(4.0);
                                stat_pill(ui, "↑", ea, enemy_col(&EnemyType::Archer));
                            });
                            ui.add_space(3.0);
                            ui.label(egui::RichText::new(
                                format!("Killed: {}  Lost: {}  Score: {}", self.game.combat.killed_enemies, self.game.combat.lost_units, self.game.combat.score))
                                .size(10.0).color(t2()));
                        });
                    });
                });
            });

        // CENTRAL VIEWPORT
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(6, 8, 12)))
            .show(ctx, |ui| {
                let avail = ui.available_rect_before_wrap();
                self.viewport_rect = avail;

                let (response, painter) = ui.allocate_painter(avail.size(), egui::Sense::click_and_drag());
                let vp_origin = avail.min;

                // Game input
                let inp = ui.input(|i| i.clone());
                handle_input(&mut self.game, &inp, vp_origin);

                // Hover tile
                self.hover_tile = inp.pointer.hover_pos().map(|p| {
                    let local = Pos2::new(p.x - vp_origin.x, p.y - vp_origin.y);
                    let world = self.game.camera.screen_to_world(local);
                    ((world.x / TILE_SIZE) as i32, (world.y / TILE_SIZE) as i32)
                });

                // Draw the game world
                draw_scene(&self.game, response.rect, painter);

                // Box-select overlay
                if let Some(ws) = self.game.input_handler.box_select_start {
                    if let Some(hover) = inp.pointer.hover_pos() {
                        let lh  = Pos2::new(hover.x - vp_origin.x, hover.y - vp_origin.y);
                        let wh  = self.game.camera.screen_to_world(lh);
                        let ss  = self.game.camera.world_to_screen(ws);
                        let se  = self.game.camera.world_to_screen(wh);
                        let sr  = Rect::from_two_pos(
                            Pos2::new(ss.x + vp_origin.x, ss.y + vp_origin.y),
                            Pos2::new(se.x + vp_origin.x, se.y + vp_origin.y),
                        );
                        ui.painter().rect_stroke(sr, 2.0, Stroke::new(1.5, Color32::from_rgba_premultiplied(85, 210, 100, 210)));
                        ui.painter().rect_filled(sr, 2.0, Color32::from_rgba_premultiplied(55, 150, 70, 22));
                    }
                }

                // Hover-tile highlight
                if let Some((tx, ty)) = self.hover_tile {
                    if Map::in_bounds(tx, ty) {
                        let tile = self.game.map.tiles[Map::idx(tx, ty)];
                        let (hc, bc) = if tile.walkable() {
                            (Color32::from_rgba_premultiplied(85, 210, 100, 30),
                             Color32::from_rgba_premultiplied(85, 210, 100, 110))
                        } else {
                            (Color32::from_rgba_premultiplied(210, 55, 55, 30),
                             Color32::from_rgba_premultiplied(210, 55, 55, 110))
                        };
                        let wc  = Pos2::new(tx as f32 * TILE_SIZE + TILE_SIZE / 2.0, ty as f32 * TILE_SIZE + TILE_SIZE / 2.0);
                        let sc  = self.game.camera.world_to_screen(wc);
                        let tsz = TILE_SIZE * self.game.camera.zoom;
                        let tr  = Rect::from_center_size(
                            Pos2::new(sc.x + vp_origin.x, sc.y + vp_origin.y),
                            Vec2::splat(tsz),
                        );
                        ui.painter().rect_filled(tr, 0.0, hc);
                        ui.painter().rect_stroke(tr, 0.0, Stroke::new(1.5, bc));
                    }
                }

                // Minimap
                if self.game.minimap_visible {
                    self.draw_minimap(ui, avail);
                }

                // HUD overlays
                let sc = self.game.selected_count();
                if sc > 0 {
                    ui.painter().text(
                        Pos2::new(avail.min.x + 12.0, avail.max.y - 12.0),
                        Align2::LEFT_BOTTOM,
                        format!("✔ {} unit{} selected", sc, if sc == 1 { "" } else { "s" }),
                        FontId::proportional(12.0), acc(),
                    );
                }
                // Zoom indicator
                let mm_offset = if self.game.minimap_visible { MINIMAP_SIZE + 20.0 } else { 14.0 };
                ui.painter().text(
                    Pos2::new(avail.max.x - mm_offset, avail.max.y - 12.0),
                    Align2::RIGHT_BOTTOM,
                    format!("⊕ {:.1}×", self.game.camera.zoom),
                    FontId::proportional(11.0), t2(),
                );
                // Coordinates
                if let Some((tx, ty)) = self.hover_tile {
                    ui.painter().text(
                        Pos2::new(avail.min.x + 10.0, avail.min.y + 10.0),
                        Align2::LEFT_TOP,
                        format!("({},{})", tx, ty),
                        FontId::proportional(10.0), t2(),
                    );
                }
                // Paused banner
                if self.game.paused {
                    ui.painter().text(
                        avail.center(), Align2::CENTER_CENTER,
                        "⏸  PAUSED",
                        FontId::proportional(32.0),
                        Color32::from_rgba_premultiplied(255, 255, 255, 210),
                    );
                }
                // Wave incoming flash (last 5 s)
                let wc = self.game.wave_countdown;
                if wc < 5.0 {
                    let a = ((wc * 3.0).sin().abs() * 210.0) as u8;
                    ui.painter().text(
                        Pos2::new(avail.center().x, avail.center().y - 65.0),
                        Align2::CENTER_CENTER,
                        format!("⚠ WAVE {} IN {:.0}s", self.game.wave_number + 1, wc.ceil()),
                        FontId::proportional(24.0),
                        Color32::from_rgba_premultiplied(220, 60, 60, a),
                    );
                }
            });

        // HELP MODAL
        if self.show_help {
            egui::Window::new("Help — Strategy RTS Engine")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(egui::Frame::window(&ctx.style()).fill(bg2()).stroke(Stroke::new(1.5, bord())))
                .show(ctx, |ui| {
                    ui.set_min_width(340.0);

                    sec(ui, "CONTROLS");   ui.add_space(6.0);
                    key_hint(ui, "Left Click (unit)",   "Select / deselect");
                    key_hint(ui, "Left Click (ground)", "Move selected units");
                    key_hint(ui, "Right Drag",          "Pan camera");
                    key_hint(ui, "Scroll Wheel",        "Zoom in / out");
                    key_hint(ui, "Drag on map",         "Box-select units");
                    key_hint(ui, "A",                   "Select all units");
                    key_hint(ui, "S",                   "Stop selected units");
                    key_hint(ui, "Esc",                 "Deselect all");
                    key_hint(ui, "⏸ button",            "Pause / resume");
                    key_hint(ui, "Double-click list",   "Focus camera on unit");

                    ui.add_space(8.0);
                    sec(ui, "FRIENDLY UNITS"); ui.add_space(6.0);
                    key_hint(ui, "⚔ Warrior", "120 HP · melee (1.6 tile range) · slow");
                    key_hint(ui, "🏹 Archer",  "70 HP  · ranged (6 tile range)  · medium");
                    key_hint(ui, "◈ Scout",   "90 HP  · melee (2 tile range)   · fast");

                    ui.add_space(8.0);
                    sec(ui, "ENEMIES"); ui.add_space(6.0);
                    key_hint(ui, "Grunt",  "60 HP · melee · patrols and chases");
                    key_hint(ui, "Brute",  "160 HP · heavy melee · slow, hard to kill");
                    key_hint(ui, "Archer", "45 HP · ranged (7 tiles) · keeps distance");

                    ui.add_space(8.0);
                    sec(ui, "TERRAIN"); ui.add_space(6.0);
                    key_hint(ui, "Road",              "Move cost 1 — fastest path");
                    key_hint(ui, "Grass / D.Grass",   "Move cost 2 — normal");
                    key_hint(ui, "Sand",               "Move cost 3 — slow");
                    key_hint(ui, "Wall/Water/Forest",  "Impassable — blocks all movement");

                    ui.add_space(10.0);
                    if ui.button(egui::RichText::new("  Close  ").color(acc())).clicked() {
                        self.show_help = false;
                    }
                });
        }

        ctx.request_repaint();
    }
}

// Minimap
impl RtsApp {
    fn draw_minimap(&self, ui: &mut egui::Ui, avail: Rect) {
        let map_rect = Rect::from_min_size(
            Pos2::new(avail.max.x - MINIMAP_SIZE - 10.0, avail.max.y - MINIMAP_SIZE - 10.0),
            Vec2::splat(MINIMAP_SIZE),
        );
        let p = ui.painter();

        p.rect_filled(map_rect, 5.0, Color32::from_rgba_premultiplied(7, 9, 14, 215));
        p.rect_stroke(map_rect, 5.0, Stroke::new(1.5, bord()));

        let sx = MINIMAP_SIZE / (MAP_W as f32 * TILE_SIZE);
        let sy = MINIMAP_SIZE / (MAP_H as f32 * TILE_SIZE);

        // Terrain
        for ty in 0..MAP_H {
            for tx in 0..MAP_W {
                let tile = self.game.map.tiles[Map::idx(tx, ty)];
                let col  = match tile {
                    Tile::Grass     => Color32::from_rgb(38,  82,  32),
                    Tile::DarkGrass => Color32::from_rgb(26,  60,  22),
                    Tile::Wall      => Color32::from_rgb(78,  72,  68),
                    Tile::Water     => Color32::from_rgb(28,  75, 155),
                    Tile::Sand      => Color32::from_rgb(175, 155,  96),
                    Tile::Forest    => Color32::from_rgb(20,  50,  15),
                    Tile::Road      => Color32::from_rgb(108,  93,  70),
                };
                let px = map_rect.min.x + tx as f32 * TILE_SIZE * sx;
                let py = map_rect.min.y + ty as f32 * TILE_SIZE * sy;
                p.rect_filled(
                    Rect::from_min_size(Pos2::new(px, py), Vec2::new((TILE_SIZE * sx).ceil() + 0.5, (TILE_SIZE * sy).ceil() + 0.5)),
                    0.0, col,
                );
            }
        }

        // Projectiles
        for proj in &self.game.projectiles {
            let mx  = map_rect.min.x + proj.pos.x * sx;
            let my  = map_rect.min.y + proj.pos.y * sy;
            let col = match proj.owner {
                crate::engine::projectile::ProjOwner::Player => Color32::from_rgb(80, 220, 100),
                crate::engine::projectile::ProjOwner::Enemy  => Color32::from_rgb(220, 80, 80),
            };
            p.circle_filled(Pos2::new(mx, my), 2.0, col);
        }

        // Friendly units
        for u in &self.game.units {
            let mx  = map_rect.min.x + u.pixel_x * sx;
            let my  = map_rect.min.y + u.pixel_y * sy;
            let col = unit_col(&u.unit_type);
            p.circle_filled(Pos2::new(mx, my), if u.selected { 4.0 } else { 2.5 }, col);
            if u.selected { p.circle_stroke(Pos2::new(mx, my), 5.0, Stroke::new(1.0, Color32::from_rgb(255, 215, 50))); }
        }

        // Enemies
        for e in &self.game.enemies {
            if e.dead { continue; }
            let mx  = map_rect.min.x + e.pixel_x * sx;
            let my  = map_rect.min.y + e.pixel_y * sy;
            p.circle_filled(Pos2::new(mx, my), 2.5, dng());
        }

        // Viewport rect
        let vm  = self.game.camera.screen_to_world(Pos2::ZERO);
        let vx2 = self.game.camera.screen_to_world(Pos2::new(self.viewport_rect.width(), self.viewport_rect.height()));
        let vr  = Rect::from_two_pos(
            Pos2::new(map_rect.min.x + vm.x  * sx, map_rect.min.y + vm.y  * sy),
            Pos2::new(map_rect.min.x + vx2.x * sx, map_rect.min.y + vx2.y * sy),
        );
        p.rect_stroke(vr, 0.0, Stroke::new(1.5, Color32::from_rgba_premultiplied(255, 255, 255, 175)));

        p.text(
            Pos2::new(map_rect.min.x + MINIMAP_SIZE / 2.0, map_rect.min.y + 7.0),
            Align2::CENTER_TOP, "MINIMAP", FontId::proportional(8.5), t2(),
        );
    }
}


fn unit_icon(t: &UnitType) -> &'static str {
    match t { UnitType::Warrior => "⚔", UnitType::Archer => "🏹", UnitType::Scout => "◈" }
}
fn enemy_icon(t: &EnemyType) -> &'static str {
    match t { EnemyType::Grunt => "✕", EnemyType::Brute => "◉", EnemyType::Archer => "↑" }
}