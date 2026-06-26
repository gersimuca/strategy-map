pub const MAP_W: i32 = 50;
pub const MAP_H: i32 = 50;

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    Grass,
    Wall,
}

pub struct Map {
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn new() -> Self {
        let mut tiles = vec![Tile::Grass; (MAP_W * MAP_H) as usize];

        // sample wall
        for x in 10..20 {
            tiles[(15 * MAP_W + x) as usize] = Tile::Wall;
        }

        Self { tiles }
    }

    pub fn idx(x: i32, y: i32) -> usize {
        (y * MAP_W + x) as usize
    }

    pub fn in_bounds(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < MAP_W && y < MAP_H
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        Self::in_bounds(x, y) && self.tiles[Self::idx(x, y)] != Tile::Wall
    }
}
