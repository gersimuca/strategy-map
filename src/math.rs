use egui::Pos2;

pub fn distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

pub fn tile_center(tile: (i32, i32), tile_size: f32) -> Pos2 {
    Pos2::new(
        tile.0 as f32 * tile_size + tile_size / 2.0,
        tile.1 as f32 * tile_size + tile_size / 2.0,
    )
}

pub fn world_to_tile(p: Pos2, tile_size: f32) -> (i32, i32) {
    ((p.x / tile_size) as i32, (p.y / tile_size) as i32)
}