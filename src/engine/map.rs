pub const MAP_W: i32 = 60;
pub const MAP_H: i32 = 60;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tile {
    Grass,
    DarkGrass,
    Wall,
    Water,
    Sand,
    Forest,
    Road,
}

impl Tile {
    pub fn walkable(&self) -> bool {
        match self {
            Tile::Grass | Tile::DarkGrass | Tile::Sand | Tile::Road => true,
            Tile::Wall | Tile::Water | Tile::Forest => false,
        }
    }

    pub fn move_cost(&self) -> i32 {
        match self {
            Tile::Road => 1,
            Tile::Grass | Tile::DarkGrass => 2,
            Tile::Sand => 3,
            _ => 999,
        }
    }
}

pub struct Map {
    pub tiles: Vec<Tile>,
    pub width: i32,
    pub height: i32,
}

impl Map {
    pub fn new() -> Self {
        let mut tiles = vec![Tile::Grass; (MAP_W * MAP_H) as usize];

        // Add dark grass patches for visual variety
        let dark_patches = [
            (5, 5, 8, 8), (30, 2, 8, 5), (45, 15, 6, 6),
            (20, 35, 10, 8), (50, 40, 8, 10), (2, 45, 7, 7),
        ];
        for (ox, oy, w, h) in dark_patches {
            for dy in 0..h {
                for dx in 0..w {
                    let x = ox + dx;
                    let y = oy + dy;
                    if Self::in_bounds(x, y) {
                        tiles[Self::idx(x, y)] = Tile::DarkGrass;
                    }
                }
            }
        }

        // Horizontal wall barrier
        for x in 10..20 {
            if Self::in_bounds(x, 15) {
                tiles[Self::idx(x, 15)] = Tile::Wall;
            }
        }

        // Vertical wall
        for y in 5..25 {
            if Self::in_bounds(25, y) {
                tiles[Self::idx(25, y)] = Tile::Wall;
            }
        }

        // L-shaped wall
        for x in 35..48 {
            if Self::in_bounds(x, 25) {
                tiles[Self::idx(x, 25)] = Tile::Wall;
            }
        }
        for y in 25..38 {
            if Self::in_bounds(35, y) {
                tiles[Self::idx(35, y)] = Tile::Wall;
            }
        }

        // Water lake
        for dy in 0..6 {
            for dx in 0..10 {
                let x = 38 + dx;
                let y = 5 + dy;
                if Self::in_bounds(x, y) {
                    tiles[Self::idx(x, y)] = Tile::Water;
                }
            }
        }

        // Sand beach around water
        for dy in -1..8 {
            for dx in -1..12 {
                let x = 37 + dx;
                let y = 4 + dy;
                if Self::in_bounds(x, y) {
                    let idx = Self::idx(x, y);
                    if tiles[idx] == Tile::Grass {
                        tiles[idx] = Tile::Sand;
                    }
                }
            }
        }

        // Forest cluster
        let forest_tiles = [
            (5, 30), (6, 30), (7, 30), (5, 31), (6, 31), (7, 31),
            (5, 32), (6, 32), (8, 30), (8, 31),
            (15, 40), (16, 40), (17, 40), (15, 41), (16, 41),
        ];
        for (x, y) in forest_tiles {
            if Self::in_bounds(x, y) {
                tiles[Self::idx(x, y)] = Tile::Forest;
            }
        }

        // Road network
        // Horizontal road
        for x in 0..MAP_W {
            if Self::in_bounds(x, 8) && tiles[Self::idx(x, 8)] == Tile::Grass {
                tiles[Self::idx(x, 8)] = Tile::Road;
            }
        }
        // Vertical road
        for y in 0..MAP_H {
            if Self::in_bounds(3, y) && tiles[Self::idx(3, y)] == Tile::Grass {
                tiles[Self::idx(3, y)] = Tile::Road;
            }
        }

        Self { tiles, width: MAP_W, height: MAP_H }
    }

    pub fn idx(x: i32, y: i32) -> usize {
        (y * MAP_W + x) as usize
    }

    pub fn in_bounds(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < MAP_W && y < MAP_H
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        Self::in_bounds(x, y) && self.tiles[Self::idx(x, y)].walkable()
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<Tile> {
        if Self::in_bounds(x, y) {
            Some(self.tiles[Self::idx(x, y)])
        } else {
            None
        }
    }
}