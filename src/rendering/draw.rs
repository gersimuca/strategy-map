use eframe::egui::{Painter, Rect, Color32, Pos2, Vec2};
use crate::engine::Game;
use crate::engine::map::{MAP_W, MAP_H, Tile};

pub fn draw_scene(game: &Game, rect: Rect, painter: Painter) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 20, 25));

    let tile_size = 32.0;

    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let tile = game.map.tiles[crate::engine::map::Map::idx(x, y)];

            let color = match tile {
                Tile::Grass => Color32::from_rgb(40, 90, 40),
                Tile::Wall => Color32::from_rgb(80, 80, 80),
            };

            let pos = Pos2::new(
                x as f32 * tile_size,
                y as f32 * tile_size,
            );

            painter.rect_filled(
                Rect::from_min_size(pos, Vec2::splat(tile_size)),
                0.0,
                color,
            );
        }
    }

    for u in &game.units {
        let pos = Pos2::new(
            u.x as f32 * tile_size + 16.0,
            u.y as f32 * tile_size + 16.0,
        );

        painter.circle_filled(
            pos,
            10.0,
            if u.selected { Color32::YELLOW } else { Color32::RED },
        );
    }
}
