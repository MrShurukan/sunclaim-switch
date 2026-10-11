use crate::tile::{Tile, TilePos, TILES_ARRAY_SIZE, TILES_ARRAY_X_DIM};

pub type TileArray = [Tile; TILES_ARRAY_SIZE];

/// Platform-independent game data and rules. Rendering and input live in rust_game.
pub struct World {
    tiles: TileArray,
}

impl World {
    pub fn new() -> Self {
        Self {
            tiles: core::array::from_fn(|index| {
                let x = index % TILES_ARRAY_X_DIM;
                let y = index / TILES_ARRAY_X_DIM;

                Tile::new(TilePos::new(x, y).expect("Invalid world tile constructor"))
            })
        }
    }
}
