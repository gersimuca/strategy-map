use eframe::egui::{Color32, FontId, Align2, Painter, Pos2, Rect, Shape, Stroke, Vec2};
use crate::engine::Game;
use crate::engine::map::{MAP_W, MAP_H, Tile};
use crate::engine::unit::{Unit, UnitType};
use crate::engine::enemy::{Enemy, EnemyType};
use crate::engine::projectile::{Projectile, ProjOwner};
use crate::math::tile_center;

const TS: f32 = 32.0;

// Colour helpers
#[inline] fn c(r: u8, g: u8, b: u8)       -> Color32 { Color32::from_rgb(r, g, b) }
#[inline] fn ca(r: u8, g: u8, b: u8, a: u8) -> Color32 { Color32::from_rgba_premultiplied(r, g, b, a) }

fn lc(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    ca(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        255,
    )
}

fn wa(col: Color32, a: u8) -> Color32 { ca(col.r(), col.g(), col.b(), a) }

// Public entry
pub fn draw_scene(game: &Game, rect: Rect, painter: Painter) {
    painter.rect_filled(rect, 0.0, c(10, 12, 18));
    draw_tiles(game, &painter);
    draw_path_overlays(game, &painter);
    draw_effects_particles(game, &painter);
    // Draw enemies and units sorted back-to-front together
    draw_entities(game, &painter);
    draw_projectiles(game, &painter);
    draw_click_rings(game, &painter);
    draw_damage_numbers(game, &painter);
}

// TILES
fn draw_tiles(game: &Game, p: &Painter) {
    let z    = game.camera.zoom;
    let time = game.time;

    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let tile  = game.map.tiles[crate::engine::map::Map::idx(x, y)];
            let cw    = tile_center((x, y), TS);
            let sc    = game.camera.world_to_screen(cw);
            let tsz   = TS * z;
            let r     = Rect::from_center_size(sc, Vec2::splat(tsz));

            // Rough frustum cull
            if sc.x < -tsz * 2.0 || sc.y < -tsz * 2.0 { continue; }

            match tile {
                Tile::Grass => {
                    let noise = ((x * 7 + y * 13) % 5) as f32 / 5.0;
                    p.rect_filled(r, 0.0, lc(c(44, 95, 38), c(54, 112, 46), noise));
                    if z > 1.2 { p.rect_stroke(r, 0.0, Stroke::new(0.5, ca(10, 10, 10, 35))); }
                }
                Tile::DarkGrass => {
                    let noise = ((x * 11 + y * 7) % 4) as f32 / 4.0;
                    p.rect_filled(r, 0.0, lc(c(28, 68, 24), c(38, 80, 30), noise));
                    if z > 1.5 {
                        for i in 0..3usize {
                            let ox = ((x as usize * 17 + i * 5) % 20) as f32 / 20.0 * tsz;
                            let oy = ((y as usize * 13 + i * 7) % 20) as f32 / 20.0 * tsz;
                            p.circle_filled(Pos2::new(r.min.x + ox, r.min.y + oy), 1.2 * z, c(20, 52, 18));
                        }
                    }
                }
                Tile::Wall => {
                    p.rect_filled(r, 0.0, c(58, 52, 46));
                    // Top bevel
                    p.rect_filled(Rect::from_min_size(r.min, Vec2::new(tsz, 3.0 * z)), 0.0, ca(148, 138, 125, 65));
                    // Bottom shadow
                    p.rect_filled(Rect::from_min_size(Pos2::new(r.min.x, r.max.y - 3.0 * z), Vec2::new(tsz, 3.0 * z)), 0.0, ca(0, 0, 0, 90));
                    // 3-D top cap
                    let cap = 6.0 * z;
                    p.rect_filled(Rect::from_min_size(Pos2::new(r.min.x, r.min.y - cap), Vec2::new(tsz, cap)), 0.0, c(80, 72, 64));
                }
                Tile::Water => {
                    let wave = (time * 1.6 + x as f32 * 0.38 + y as f32 * 0.29).sin() * 0.12 + 0.88;
                    p.rect_filled(r, 0.0, lc(c(22, 65, 135), c(38, 95, 175), wave));
                    if z > 1.0 {
                        let sh = (time * 2.1 + x as f32 * 0.52).sin() * 0.5 + 0.5;
                        if sh > 0.68 {
                            let ly = r.center().y + (sh - 0.68) * 12.0 * z;
                            p.line_segment(
                                [Pos2::new(r.min.x + 2.0, ly), Pos2::new(r.max.x - 2.0, ly)],
                                Stroke::new(z, ca(170, 215, 255, 75)),
                            );
                        }
                    }
                }
                Tile::Sand => {
                    let noise = ((x * 5 + y * 11) % 6) as f32 / 6.0;
                    p.rect_filled(r, 0.0, lc(c(188, 162, 108), c(208, 182, 128), noise));
                    if z > 1.5 {
                        for i in 0..4usize {
                            let ox = ((x as usize * 19 + i * 11) % 24) as f32 / 24.0 * tsz;
                            let oy = ((y as usize * 17 + i * 13) % 24) as f32 / 24.0 * tsz;
                            p.circle_filled(Pos2::new(r.min.x + ox, r.min.y + oy), z, ca(155, 130, 75, 55));
                        }
                    }
                }
                Tile::Forest => {
                    p.rect_filled(r, 0.0, c(20, 50, 16));
                    if z > 0.65 {
                        let sway = (time * 0.85 + x as f32 * 1.25).sin() * 1.6 * z;
                        let cx2  = r.center().x + sway * 0.3;
                        let cy2  = r.center().y;
                        p.circle_filled(Pos2::new(cx2, cy2),                          9.5 * z, c(16, 42, 12));
                        p.circle_filled(Pos2::new(cx2 + sway * 0.2, cy2 - 2.5 * z),  7.0 * z, c(32, 76, 22));
                        p.circle_filled(Pos2::new(cx2 + sway * 0.1, cy2 - 5.0 * z),  4.2 * z, c(50, 105, 35));
                    } else {
                        p.rect_filled(r, 0.0, c(30, 70, 22));
                    }
                }
                Tile::Road => {
                    p.rect_filled(r, 0.0, c(118, 103, 78));
                    if z > 0.9 {
                        p.rect_filled(
                            Rect::from_center_size(r.center(), Vec2::new(tsz * 0.14, tsz)),
                            0.0, ca(155, 142, 108, 110),
                        );
                    }
                }
            }
        }
    }
}

// PATH OVERLAYS
fn draw_path_overlays(game: &Game, p: &Painter) {
    for u in game.units.iter().filter(|u| u.selected && !u.path.is_empty()) {
        let mut prev = u.world_pos();
        for &(px, py) in &u.path {
            let nw = Pos2::new(px as f32 * TS + TS / 2.0, py as f32 * TS + TS / 2.0);
            p.line_segment(
                [game.camera.world_to_screen(prev), game.camera.world_to_screen(nw)],
                Stroke::new(1.8, ca(80, 215, 90, 110)),
            );
            p.circle_filled(game.camera.world_to_screen(nw), 2.5, ca(80, 215, 90, 150));
            prev = nw;
        }
        // Goal ring
        if let Some(&(gx, gy)) = u.path.last() {
            let gs    = game.camera.world_to_screen(Pos2::new(gx as f32 * TS + TS / 2.0, gy as f32 * TS + TS / 2.0));
            let pulse = (game.time * 3.2).sin() * 0.3 + 1.0;
            p.circle_stroke(gs, 8.0 * pulse, Stroke::new(2.0, ca(90, 255, 100, 170)));
            p.circle_filled(gs, 3.0, c(90, 255, 100));
        }
    }
}

// PARTICLES
fn draw_effects_particles(game: &Game, p: &Painter) {
    let z = game.camera.zoom;
    for pt in &game.effects.particles {
        let sc  = game.camera.world_to_screen(pt.pos);
        let a   = (pt.alpha_factor() * pt.color[3] as f32) as u8;
        p.circle_filled(sc, pt.size * z * 0.7, ca(pt.color[0], pt.color[1], pt.color[2], a));
    }
}

fn draw_click_rings(game: &Game, p: &Painter) {
    let z = game.camera.zoom;
    for ci in &game.effects.click_indicators {
        let sc  = game.camera.world_to_screen(ci.pos);
        let a   = (ci.alpha() * ci.color[3] as f32) as u8;
        let col = ca(ci.color[0], ci.color[1], ci.color[2], a);
        p.circle_stroke(sc, ci.radius * z,        Stroke::new(2.0, col));
        let a2 = (ci.alpha() * ci.alpha() * ci.color[3] as f32) as u8;
        p.circle_stroke(sc, ci.radius * z * 0.5,  Stroke::new(1.5, ca(ci.color[0], ci.color[1], ci.color[2], a2)));
    }
}

// PROJECTILES
fn draw_projectiles(game: &Game, p: &Painter) {
    let z = game.camera.zoom;
    for proj in &game.projectiles {
        // Trail
        for (i, (tpos, alpha)) in proj.trail.iter().enumerate() {
            let sc  = game.camera.world_to_screen(*tpos);
            let a   = (*alpha * 180.0) as u8;
            let sz  = (i as f32 / proj.trail.len().max(1) as f32) * proj.radius * z;
            match proj.owner {
                ProjOwner::Player => p.circle_filled(sc, sz, ca(60,  200, 100, a)),
                ProjOwner::Enemy  => p.circle_filled(sc, sz, ca(220, 60,  60,  a)),
            };
        }
        // Head
        let sc = game.camera.world_to_screen(proj.pos);
        let (head, glow) = match proj.owner {
            ProjOwner::Player => (c(90, 255, 120), ca(60,  200, 90,  80)),
            ProjOwner::Enemy  => (c(255, 80,  60),  ca(200, 50,  50,  80)),
        };
        p.circle_filled(sc, proj.radius * z * 1.9, glow);
        p.circle_filled(sc, proj.radius * z,        head);
    }
}

// ENTITIES – pseudo-3D units AND enemies, sorted back-to-front
enum EntityRef<'a> {
    Unit(&'a Unit),
    Enemy(&'a Enemy),
}

fn entity_py(e: &EntityRef) -> f32 {
    match e { EntityRef::Unit(u) => u.pixel_y, EntityRef::Enemy(e) => e.pixel_y }
}

fn draw_entities(game: &Game, p: &Painter) {
    // Collect references
    let mut all: Vec<EntityRef> = vec![];
    for u in &game.units  { all.push(EntityRef::Unit(u)); }
    for e in &game.enemies { all.push(EntityRef::Enemy(e)); }
    all.sort_by(|a, b| entity_py(a).partial_cmp(&entity_py(b)).unwrap());

    for ent in &all {
        match ent {
            EntityRef::Unit(u)  => draw_unit(game, p, u),
            EntityRef::Enemy(e) => draw_enemy(game, p, e),
        }
    }
}

// Pseudo-3D body helper
fn draw_3d_body(
    p:          &Painter,
    sc:         Pos2,
    radius:     f32,
    height_px:  f32,
    body_col:   Color32,
    hi_col:     Color32,
    flash:      f32,
    fade_alpha: f32,
) {
    let r  = radius;
    let h  = height_px;
    let a8 = (fade_alpha * 255.0).clamp(0.0, 255.0) as u8;

    // Ground shadow ellipse
    p.circle_filled(
        Pos2::new(sc.x + 2.5, sc.y + 3.5 + h * 0.45),
        r * 0.9,
        ca(0, 0, 0, (a8 as f32 * 0.4) as u8),
    );

    // Column side (darker body colour, drawn at y offset)
    let side = lc(body_col, c(0, 0, 0), 0.45);
    let steps = ((h / 2.0) as i32).max(1);
    for s in 0..=steps {
        let t   = s as f32 / steps as f32;
        let col = lc(side, body_col, t);
        p.circle_filled(
            Pos2::new(sc.x, sc.y + h * (1.0 - t)),
            r * (0.85 + t * 0.15),
            wa(col, a8),
        );
    }

    // Top disc
    let top = Pos2::new(sc.x, sc.y - h * 0.05);
    p.circle_filled(top, r, wa(body_col, a8));

    // Specular highlight
    let hl_a = if flash > 0.0 { (flash * 255.0) as u8 } else { 165 };
    p.circle_filled(
        Pos2::new(top.x - r * 0.27, top.y - r * 0.27),
        r * 0.42,
        wa(hi_col, ((hl_a as f32 * fade_alpha) as u8).min(a8)),
    );

    // Attack flash ring
    if flash > 0.0 {
        p.circle_stroke(
            top, r * 1.32,
            Stroke::new(2.5, ca(255, 255, 180, (flash * 200.0 * fade_alpha) as u8)),
        );
    }
}

// Player unit
fn draw_unit(game: &Game, p: &Painter, u: &Unit) {
    let z  = game.camera.zoom;
    let sc = game.camera.world_to_screen(u.world_pos());
    let r  = u.unit_type.radius() * z;
    let h  = u.unit_type.visual_height() * z;

    // Move trail
    for t in &u.move_effects {
        let ts = game.camera.world_to_screen(t.pos);
        p.circle_filled(ts, r * 0.3, ca(70, 170, 70, (t.alpha * 90.0) as u8));
    }

    // Selection ring
    if u.selected {
        let pulse = u.selection_pulse.sin() * 0.18 + 1.0;
        p.circle_stroke(sc, r * 1.65 * pulse, Stroke::new(2.5, ca(255, 215, 50, 200)));
        p.circle_stroke(sc, r * 1.65 * pulse + 3.5 * z, Stroke::new(1.0, ca(255, 215, 50, 55)));
    }

    let (body, hi) = match u.unit_type {
        UnitType::Warrior => (c(175, 48, 48),  c(225, 95, 95)),
        UnitType::Archer  => (c(48,  125, 48),  c(80,  178, 80)),
        UnitType::Scout   => (c(48,  95,  195), c(80,  138, 240)),
    };
    draw_3d_body(p, sc, r, h, body, hi, u.attack_flash, 1.0);

    // Class icon on top face
    let top = Pos2::new(sc.x, sc.y - h * 0.05);
    if z > 0.75 {
        let s = r * 0.52;
        match u.unit_type {
            UnitType::Warrior => {
                p.line_segment([Pos2::new(sc.x, top.y - s), Pos2::new(sc.x, top.y + s)],
                               Stroke::new(2.0 * z, ca(255, 255, 255, 190)));
                p.line_segment([Pos2::new(sc.x - s * 0.42, top.y - s * 0.18), Pos2::new(sc.x + s * 0.42, top.y - s * 0.18)],
                               Stroke::new(2.0 * z, ca(255, 255, 255, 190)));
            }
            UnitType::Archer => {
                // Arrow: vertical shaft + chevron tip
                p.line_segment([Pos2::new(sc.x, top.y - s), Pos2::new(sc.x, top.y + s * 0.4)],
                               Stroke::new(2.0 * z, ca(255, 255, 255, 190)));
                p.line_segment([Pos2::new(sc.x - s * 0.3, top.y - s * 0.5), Pos2::new(sc.x, top.y - s)],
                               Stroke::new(2.0 * z, ca(255, 255, 255, 190)));
                p.line_segment([Pos2::new(sc.x + s * 0.3, top.y - s * 0.5), Pos2::new(sc.x, top.y - s)],
                               Stroke::new(2.0 * z, ca(255, 255, 255, 190)));
            }
            UnitType::Scout => {
                let pts = vec![
                    Pos2::new(sc.x,        top.y - s),
                    Pos2::new(sc.x + s * 0.6, top.y),
                    Pos2::new(sc.x,        top.y + s),
                    Pos2::new(sc.x - s * 0.6, top.y),
                ];
                p.add(Shape::convex_polygon(pts, ca(255, 255, 255, 165), Stroke::NONE));
            }
        }
    }

    // HP bar
    if z > 0.55 {
        let bw = r * 2.3;
        let bh = 3.8 * z;
        let by = sc.y - r - h * 0.1 - 9.0 * z;
        let bx = sc.x - bw / 2.0;
        p.rect_filled(Rect::from_min_size(Pos2::new(bx - 0.5, by - 0.5), Vec2::new(bw + 1.0, bh + 1.0)), 2.0, ca(0, 0, 0, 170));
        let hp  = (u.health / u.max_health).clamp(0.0, 1.0);
        let hpc = if hp > 0.6 { c(45, 195, 55) } else if hp > 0.3 { c(215, 175, 25) } else { c(215, 55, 35) };
        if hp > 0.0 {
            p.rect_filled(Rect::from_min_size(Pos2::new(bx, by), Vec2::new(bw * hp, bh)), 2.0, hpc);
        }
    }

    // Name label at high zoom
    if z > 1.5 {
        p.text(
            Pos2::new(sc.x, sc.y + r + h * 0.1 + 8.0 * z),
            Align2::CENTER_TOP,
            u.unit_type.name(),
            FontId::proportional(9.5 * z),
            ca(200, 210, 225, 175),
        );
    }
}

// Enemy unit
fn draw_enemy(game: &Game, p: &Painter, e: &Enemy) {
    let z     = game.camera.zoom;
    let fade  = if e.dead { e.death_timer.clamp(0.0, 1.0) } else { 1.0 };
    let sc    = game.camera.world_to_screen(e.world_pos());
    let r     = e.enemy_type.radius() * z;
    let h     = e.enemy_type.visual_height() * z;

    // Attack / detect range ring
    if matches!(e.ai_state, crate::engine::enemy::AiState::Attack { .. }) {
        let range_sc = e.enemy_type.attack_range_tiles() * TS * z;
        p.circle_stroke(sc, range_sc, Stroke::new(1.0, ca(200, 40, 40, 40)));
    }

    let (body, hi) = match e.enemy_type {
        EnemyType::Grunt  => (c(135, 28, 28), c(185, 65, 65)),
        EnemyType::Brute  => (c(90,  18, 18), c(145, 40, 40)),
        EnemyType::Archer => (c(140, 60, 20), c(190, 100, 55)),
    };
    draw_3d_body(p, sc, r, h, body, hi, e.attack_flash * fade, fade);

    // Class symbol on top
    let top = Pos2::new(sc.x, sc.y - h * 0.05);
    if z > 0.75 && fade > 0.3 {
        let s  = r * 0.5;
        let wa8 = (fade * 180.0) as u8;
        match e.enemy_type {
            EnemyType::Grunt => {
                // X mark
                p.line_segment([Pos2::new(sc.x - s, top.y - s), Pos2::new(sc.x + s, top.y + s)], Stroke::new(2.0 * z, ca(255, 200, 180, wa8)));
                p.line_segment([Pos2::new(sc.x + s, top.y - s), Pos2::new(sc.x - s, top.y + s)], Stroke::new(2.0 * z, ca(255, 200, 180, wa8)));
            }
            EnemyType::Brute => {
                // Club: circle + rod
                p.circle_filled(Pos2::new(sc.x, top.y - s * 0.3), s * 0.65, ca(255, 220, 190, wa8));
                p.line_segment([Pos2::new(sc.x, top.y - s * 0.3), Pos2::new(sc.x, top.y + s * 0.7)], Stroke::new(3.5 * z, ca(255, 220, 190, wa8)));
            }
            EnemyType::Archer => {
                // Bow arc (8-segment polyline)
                let arc: Vec<Pos2> = (0..=8).map(|i| {
                    let t = i as f32 / 8.0 * std::f32::consts::PI;
                    Pos2::new(sc.x - s * t.sin() * 0.7, top.y - s * (t.cos() - 1.0) * 0.5)
                }).collect();
                for w in arc.windows(2) {
                    p.line_segment([w[0], w[1]], Stroke::new(1.8 * z, ca(255, 210, 160, wa8)));
                }
            }
        }
    }

    // HP bar
    if z > 0.5 && fade > 0.3 {
        let bw = r * 2.4;
        let bh = 3.8 * z;
        let by = sc.y - r - h * 0.1 - 9.0 * z;
        let bx = sc.x - bw / 2.0;
        p.rect_filled(Rect::from_min_size(Pos2::new(bx - 0.5, by - 0.5), Vec2::new(bw + 1.0, bh + 1.0)), 2.0, ca(0, 0, 0, 160));
        let hp  = (e.health / e.max_health).clamp(0.0, 1.0);
        let hpc = if hp > 0.5 { c(185, 45, 45) } else { c(215, 120, 25) };
        if hp > 0.0 {
            p.rect_filled(Rect::from_min_size(Pos2::new(bx, by), Vec2::new(bw * hp, bh)), 2.0, hpc);
        }
    }

    // Chase / aggro ring
    if matches!(e.ai_state, crate::engine::enemy::AiState::Chase { .. }) && z > 0.7 && fade > 0.5 {
        let pulse = (game.time * 5.0).sin() * 0.2 + 1.0;
        p.circle_stroke(sc, r * 1.75 * pulse, Stroke::new(1.5, ca(255, 80, 40, 140)));
    }

    // Name label
    if z > 1.4 && fade > 0.5 {
        p.text(
            Pos2::new(sc.x, sc.y + r + h * 0.1 + 8.0 * z),
            Align2::CENTER_TOP,
            e.enemy_type.name(),
            FontId::proportional(9.0 * z),
            ca(220, 140, 140, 165),
        );
    }
}


// FLOATING DAMAGE NUMBERS
fn draw_damage_numbers(game: &Game, p: &Painter) {
    let z = game.camera.zoom;

    for u in &game.units {
        for dn in &u.damage_numbers {
            let sc  = game.camera.world_to_screen(dn.pos);
            let a   = (dn.lifetime / 1.1 * 230.0).clamp(0.0, 230.0) as u8;
            let col = if dn.is_crit { ca(255, 80,  40, a) } else { ca(255, 200, 60, a) };
            let sz  = if dn.is_crit { 14.0 * z } else { 11.0 * z };
            p.text(sc, Align2::CENTER_CENTER, format!("{:.0}", dn.value), FontId::proportional(sz), col);
        }
    }

    for e in &game.enemies {
        for dn in &e.damage_numbers {
            let sc  = game.camera.world_to_screen(dn.pos);
            let a   = (dn.lifetime / 1.1 * 230.0).clamp(0.0, 230.0) as u8;
            let col = if dn.is_crit { ca(255, 255, 80, a) } else { ca(230, 230, 230, a) };
            let sz  = if dn.is_crit { 14.0 * z } else { 11.0 * z };
            p.text(sc, Align2::CENTER_CENTER, format!("{:.0}", dn.value), FontId::proportional(sz), col);
        }
    }
}