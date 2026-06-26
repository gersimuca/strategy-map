use eframe::egui::{Painter, Rect, Color32, Pos2, Vec2};
use crate::engine::Game;
use crate::engine::map::{MAP_W, MAP_H, Tile};
use crate::math::tile_center;

pub fn draw_scene(game: &Game, rect: Rect, painter: Painter) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(18, 18, 22));

    let tile_size = 32.0;

    // --- MAP ---
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let tile = game.map.tiles[crate::engine::map::Map::idx(x, y)];

            let color = match tile {
                Tile::Grass => Color32::from_rgb(40, 90, 40),
                Tile::Wall => Color32::from_rgb(80, 80, 80),
            };

            let center = tile_center((x, y), tile_size);

            let screen = game.camera.world_to_screen(center);

            painter.rect_filled(
                Rect::from_center_size(screen, Vec2::splat(tile_size)),
                0.0,
                color,
            );
        }
    }

    // --- UNITS ---
    for u in &game.units {
        let center = tile_center((u.x, u.y), tile_size);
        let screen = game.camera.world_to_screen(center);

        painter.circle_filled(
            screen,
            10.0,
            if u.selected {
                Color32::YELLOW
            } else {
                Color32::RED
            },
        );
    }
}
