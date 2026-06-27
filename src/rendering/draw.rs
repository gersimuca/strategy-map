use eframe::egui::{Painter, Rect, Color32, Vec2, Pos2, Stroke, FontId, Align2, Shape};
use crate::engine::Game;
use crate::engine::map::{MAP_W, MAP_H, Tile};
use crate::math::tile_center;

const TILE_SIZE: f32 = 32.0;

fn col(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

fn cola(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_premultiplied(r, g, b, a)
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}

pub fn draw_scene(game: &Game, rect: Rect, painter: Painter) {
    // Background
    painter.rect_filled(rect, 0.0, col(12, 14, 20));

    draw_tiles(game, &painter);
    draw_path_overlays(game, &painter);
    draw_effects(game, &painter);
    draw_units(game, &painter);
    draw_box_select_preview(game, &painter);
}

fn draw_tiles(game: &Game, painter: &Painter) {
    let time = game.time;

    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let tile = game.map.tiles[crate::engine::map::Map::idx(x, y)];
            let center_world = tile_center((x, y), TILE_SIZE);
            let screen = game.camera.world_to_screen(center_world);
            let half = TILE_SIZE * game.camera.zoom / 2.0 + 0.5;

            // Cull tiles outside viewport (approximate)
            // Skip tiles far offscreen
            if screen.x < -half * 2.0 || screen.y < -half * 2.0 {
                continue;
            }

            let size = Vec2::splat(TILE_SIZE * game.camera.zoom);
            let tile_rect = Rect::from_center_size(screen, size);

            match tile {
                Tile::Grass => {
                    // Subtle noise-based color variation using tile position
                    let noise = ((x * 7 + y * 13) % 5) as f32 / 5.0;
                    let base = lerp_color(col(48, 100, 42), col(56, 115, 50), noise);
                    painter.rect_filled(tile_rect, 0.0, base);

                    // Tiny grid lines
                    if game.camera.zoom > 1.2 {
                        let grid_color = cola(20, 20, 20, 40);
                        painter.rect_stroke(tile_rect, 0.0, Stroke::new(0.5, grid_color));
                    }
                }
                Tile::DarkGrass => {
                    let noise = ((x * 11 + y * 7) % 4) as f32 / 4.0;
                    let base = lerp_color(col(32, 74, 30), col(40, 85, 36), noise);
                    painter.rect_filled(tile_rect, 0.0, base);
                    // Small dots for texture
                    if game.camera.zoom > 1.5 {
                        for i in 0..3 {
                            let ox = ((x * 17 + i * 5) % 20) as f32 / 20.0 * TILE_SIZE * game.camera.zoom;
                            let oy = ((y * 13 + i * 7) % 20) as f32 / 20.0 * TILE_SIZE * game.camera.zoom;
                            let dot_pos = Pos2::new(tile_rect.min.x + ox, tile_rect.min.y + oy);
                            painter.circle_filled(dot_pos, 1.2 * game.camera.zoom, col(25, 60, 22));
                        }
                    }
                }
                Tile::Wall => {
                    painter.rect_filled(tile_rect, 0.0, col(60, 55, 50));
                    // Stone texture
                    let highlight = cola(140, 130, 120, 60);
                    let shadow = cola(0, 0, 0, 80);
                    // Top-left highlight
                    painter.rect_filled(
                        Rect::from_min_size(tile_rect.min, Vec2::new(tile_rect.width(), 2.5 * game.camera.zoom)),
                        0.0, highlight,
                    );
                    // Bottom-right shadow
                    painter.rect_filled(
                        Rect::from_min_size(
                            Pos2::new(tile_rect.min.x, tile_rect.max.y - 2.5 * game.camera.zoom),
                            Vec2::new(tile_rect.width(), 2.5 * game.camera.zoom)
                        ),
                        0.0, shadow,
                    );
                }
                Tile::Water => {
                    // Animated water
                    let wave = (time * 1.5 + x as f32 * 0.4 + y as f32 * 0.3).sin() * 0.1 + 0.9;
                    let water_col = lerp_color(
                        col(25, 70, 140),
                        col(40, 100, 180),
                        wave,
                    );
                    painter.rect_filled(tile_rect, 0.0, water_col);

                    // Shimmer lines
                    if game.camera.zoom > 1.0 {
                        let shimmer_t = (time * 2.0 + x as f32 * 0.5).sin() * 0.5 + 0.5;
                        if shimmer_t > 0.7 {
                            let line_y = tile_rect.center().y + (shimmer_t - 0.7) * 10.0 * game.camera.zoom;
                            painter.line_segment(
                                [Pos2::new(tile_rect.min.x + 2.0, line_y), Pos2::new(tile_rect.max.x - 2.0, line_y)],
                                Stroke::new(1.0 * game.camera.zoom, cola(180, 220, 255, 80)),
                            );
                        }
                    }
                }
                Tile::Sand => {
                    let noise = ((x * 5 + y * 11) % 6) as f32 / 6.0;
                    let sand_col = lerp_color(col(190, 165, 110), col(210, 185, 130), noise);
                    painter.rect_filled(tile_rect, 0.0, sand_col);
                    if game.camera.zoom > 1.5 {
                        let dot_alpha = 50u8;
                        for i in 0..4 {
                            let ox = ((x * 19 + i * 11) % 24) as f32 / 24.0 * tile_rect.width();
                            let oy = ((y * 17 + i * 13) % 24) as f32 / 24.0 * tile_rect.height();
                            painter.circle_filled(
                                Pos2::new(tile_rect.min.x + ox, tile_rect.min.y + oy),
                                1.0 * game.camera.zoom,
                                cola(160, 135, 80, dot_alpha),
                            );
                        }
                    }
                }
                Tile::Forest => {
                    painter.rect_filled(tile_rect, 0.0, col(22, 55, 18));
                    // Tree canopy circles
                    if game.camera.zoom > 0.7 {
                        let sway = (time * 0.8 + x as f32 * 1.2).sin() * 1.5 * game.camera.zoom;
                        let center = tile_rect.center();
                        // Dark base
                        painter.circle_filled(center, 9.0 * game.camera.zoom, col(18, 45, 14));
                        // Main canopy
                        painter.circle_filled(
                            Pos2::new(center.x + sway * 0.3, center.y - 2.0 * game.camera.zoom),
                            7.0 * game.camera.zoom,
                            col(35, 80, 25),
                        );
                        // Top highlight
                        painter.circle_filled(
                            Pos2::new(center.x + sway * 0.2, center.y - 4.0 * game.camera.zoom),
                            4.0 * game.camera.zoom,
                            col(55, 110, 40),
                        );
                    } else {
                        painter.rect_filled(tile_rect, 0.0, col(30, 70, 22));
                    }
                }
                Tile::Road => {
                    painter.rect_filled(tile_rect, 0.0, col(120, 105, 80));
                    // Road lane lines
                    if game.camera.zoom > 1.0 {
                        let center = tile_rect.center();
                        let lane_color = cola(160, 145, 110, 120);
                        painter.rect_filled(
                            Rect::from_center_size(center, Vec2::new(tile_rect.width() * 0.15, tile_rect.height())),
                            0.0, lane_color,
                        );
                    }
                }
            }
        }
    }
}

fn draw_path_overlays(game: &Game, painter: &Painter) {
    // Draw path lines for selected units
    for unit in game.units.iter().filter(|u| u.selected && !u.path.is_empty()) {
        let mut prev = unit.world_pos();
        let path_color = cola(100, 220, 100, 120);
        let dot_color = cola(80, 200, 80, 160);

        for &(tx, ty) in &unit.path {
            let next_world = Pos2::new(tx as f32 * TILE_SIZE + TILE_SIZE / 2.0, ty as f32 * TILE_SIZE + TILE_SIZE / 2.0);
            let prev_screen = game.camera.world_to_screen(prev);
            let next_screen = game.camera.world_to_screen(next_world);

            painter.line_segment(
                [prev_screen, next_screen],
                Stroke::new(2.0, path_color),
            );
            painter.circle_filled(next_screen, 3.0, dot_color);
            prev = next_world;
        }

        // Destination marker
        if let Some(&(gx, gy)) = unit.path.last() {
            let goal_world = Pos2::new(gx as f32 * TILE_SIZE + TILE_SIZE / 2.0, gy as f32 * TILE_SIZE + TILE_SIZE / 2.0);
            let goal_screen = game.camera.world_to_screen(goal_world);
            let pulse = (game.time * 3.0).sin() * 0.3 + 0.7;
            painter.circle_stroke(goal_screen, 8.0 * pulse, Stroke::new(2.0, cola(100, 255, 100, 180)));
            painter.circle_filled(goal_screen, 3.0, col(100, 255, 100));
        }
    }
}

fn draw_effects(game: &Game, painter: &Painter) {
    // Draw particles
    for p in &game.effects.particles {
        let screen = game.camera.world_to_screen(p.pos);
        let alpha = (p.alpha_factor() * p.color[3] as f32) as u8;
        let color = Color32::from_rgba_premultiplied(p.color[0], p.color[1], p.color[2], alpha);
        painter.circle_filled(screen, p.size * game.camera.zoom * 0.7, color);
    }

    // Draw click indicators (rings)
    for ci in &game.effects.click_indicators {
        let screen = game.camera.world_to_screen(ci.pos);
        let alpha = (ci.alpha() * ci.color[3] as f32) as u8;
        let color = Color32::from_rgba_premultiplied(ci.color[0], ci.color[1], ci.color[2], alpha);
        painter.circle_stroke(
            screen,
            ci.radius * game.camera.zoom,
            Stroke::new(2.0, color),
        );
        // Inner ring (smaller, faster fade)
        let inner_alpha = (ci.alpha() * ci.alpha() * ci.color[3] as f32) as u8;
        let inner_color = Color32::from_rgba_premultiplied(ci.color[0], ci.color[1], ci.color[2], inner_alpha);
        painter.circle_stroke(
            screen,
            ci.radius * game.camera.zoom * 0.5,
            Stroke::new(1.5, inner_color),
        );
    }
}

fn draw_units(game: &Game, painter: &Painter) {
    for unit in &game.units {
        let screen = game.camera.world_to_screen(unit.world_pos());
        let radius = unit.unit_type.radius() * game.camera.zoom;

        // Draw move trail effects
        for trail in &unit.move_effects {
            let trail_screen = game.camera.world_to_screen(trail.pos);
            let alpha = (trail.alpha * 120.0) as u8;
            let trail_color = Color32::from_rgba_premultiplied(80, 180, 80, alpha);
            painter.circle_filled(trail_screen, radius * 0.35, trail_color);
        }

        // Selection ring
        if unit.selected {
            let pulse = (unit.selection_pulse.sin() * 0.15 + 1.0);
            let sel_radius = radius * 1.5 * pulse;
            let sel_color = cola(255, 220, 50, 200);
            painter.circle_stroke(screen, sel_radius, Stroke::new(2.5, sel_color));
            // Outer glow
            painter.circle_stroke(screen, sel_radius + 3.0 * game.camera.zoom, Stroke::new(1.0, cola(255, 220, 50, 60)));
        }

        // Shadow
        painter.circle_filled(
            Pos2::new(screen.x + 2.0 * game.camera.zoom, screen.y + 3.0 * game.camera.zoom),
            radius * 0.85,
            cola(0, 0, 0, 80),
        );

        // Unit body color by type
        let (body_color, highlight_color) = match unit.unit_type {
            crate::engine::unit::UnitType::Warrior => (col(180, 50, 50), col(220, 90, 90)),
            crate::engine::unit::UnitType::Archer => (col(50, 130, 50), col(80, 180, 80)),
            crate::engine::unit::UnitType::Scout => (col(50, 100, 200), col(80, 140, 240)),
        };

        // Body
        painter.circle_filled(screen, radius, body_color);

        // Inner highlight (top-left)
        painter.circle_filled(
            Pos2::new(screen.x - radius * 0.25, screen.y - radius * 0.25),
            radius * 0.45,
            highlight_color,
        );

        // Type indicator (small icon-like shape)
        match unit.unit_type {
            crate::engine::unit::UnitType::Warrior => {
                // Sword indicator
                if game.camera.zoom > 0.8 {
                    let s = radius * 0.55;
                    painter.line_segment(
                        [Pos2::new(screen.x, screen.y - s), Pos2::new(screen.x, screen.y + s)],
                        Stroke::new(1.5 * game.camera.zoom, cola(255, 255, 255, 180)),
                    );
                    painter.line_segment(
                        [Pos2::new(screen.x - s * 0.4, screen.y - s * 0.2), Pos2::new(screen.x + s * 0.4, screen.y - s * 0.2)],
                        Stroke::new(1.5 * game.camera.zoom, cola(255, 255, 255, 180)),
                    );
                }
            }
            crate::engine::unit::UnitType::Archer => {
                // Bow arc
                if game.camera.zoom > 0.8 {
                    // Simple arrow pointing up
                    let s = radius * 0.5;
                    painter.line_segment(
                        [Pos2::new(screen.x, screen.y - s), Pos2::new(screen.x, screen.y + s * 0.3)],
                        Stroke::new(1.5 * game.camera.zoom, cola(255, 255, 255, 180)),
                    );
                }
            }
            crate::engine::unit::UnitType::Scout => {
                // Diamond
                if game.camera.zoom > 0.8 {
                    let s = radius * 0.45;
                    let pts = [
                        Pos2::new(screen.x, screen.y - s),
                        Pos2::new(screen.x + s * 0.6, screen.y),
                        Pos2::new(screen.x, screen.y + s),
                        Pos2::new(screen.x - s * 0.6, screen.y),
                    ];
                    painter.add(Shape::convex_polygon(
                        pts.to_vec(),
                        cola(255, 255, 255, 160),
                        Stroke::NONE,
                    ));
                }
            }
        }

        // Health bar
        if game.camera.zoom > 0.6 {
            let bar_w = radius * 2.2;
            let bar_h = 3.5 * game.camera.zoom;
            let bar_y = screen.y - radius - 7.0 * game.camera.zoom;
            let bar_x = screen.x - bar_w / 2.0;

            // Background
            painter.rect_filled(
                Rect::from_min_size(Pos2::new(bar_x - 0.5, bar_y - 0.5), Vec2::new(bar_w + 1.0, bar_h + 1.0)),
                2.0,
                cola(0, 0, 0, 160),
            );

            // Health fill
            let hp_ratio = (unit.health / unit.max_health).clamp(0.0, 1.0);
            let hp_color = if hp_ratio > 0.6 {
                col(50, 200, 50)
            } else if hp_ratio > 0.3 {
                col(220, 180, 30)
            } else {
                col(220, 60, 40)
            };
            if hp_ratio > 0.0 {
                painter.rect_filled(
                    Rect::from_min_size(Pos2::new(bar_x, bar_y), Vec2::new(bar_w * hp_ratio, bar_h)),
                    2.0,
                    hp_color,
                );
            }
        }

        // Unit label when zoomed in enough
        if game.camera.zoom > 1.5 {
            let label = unit.unit_type.name();
            painter.text(
                Pos2::new(screen.x, screen.y + radius + 8.0 * game.camera.zoom),
                Align2::CENTER_TOP,
                label,
                FontId::proportional(10.0 * game.camera.zoom),
                cola(200, 200, 200, 180),
            );
        }
    }
}

fn draw_box_select_preview(_game: &Game, _painter: &Painter) {
    // Box select preview is handled in the app layer with hover pos
}