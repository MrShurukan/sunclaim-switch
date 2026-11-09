use crate::direction::Direction;

pub const TILES_ARRAY_X_DIM: usize = 24;
pub const TILES_ARRAY_Y_DIM: usize = 12;
pub const TILES_ARRAY_SIZE: usize = TILES_ARRAY_X_DIM * TILES_ARRAY_Y_DIM;

#[derive(Debug, Clone)]
pub struct Tile {
    tile_pos: TilePos,
}

impl Tile {
    #[must_use]
    pub const fn new(tile_pos: TilePos) -> Self {
        Self { tile_pos }
    }

    #[must_use]
    pub const fn tile_pos(&self) -> TilePos {
        self.tile_pos
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Into)]
pub struct TilePos {
    x: usize,
    y: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TilePosError {
    OutOfBoundsX,
    OutOfBoundsY,
}

impl TilePos {
    /// Creates a new tile pos, ensuring it stays within bounds
    ///
    /// # Errors
    ///
    /// This function will return an error if either X or Y are out of bounds of the tile array (with a corresponding error).
    pub const fn new(x: usize, y: usize) -> Result<Self, TilePosError> {
        if x >= TILES_ARRAY_X_DIM {
            return Err(TilePosError::OutOfBoundsX);
        }

        if y >= TILES_ARRAY_Y_DIM {
            return Err(TilePosError::OutOfBoundsY);
        }

        Ok(Self { x, y })
    }

    /// Attempts to create a new [`TilePos`] shifted to a specified direction
    ///
    /// # Errors
    ///
    /// This function will return an error if the new position lands out of bounds of the tile array.
    pub fn translate(self, direction: Direction) -> Result<Self, TilePosError> {
        let (new_x, new_y) = match direction {
            Direction::Up => (
                self.x,
                self.y.checked_sub(1).ok_or(TilePosError::OutOfBoundsY)?,
            ),
            Direction::Right => (
                self.x.checked_add(1).ok_or(TilePosError::OutOfBoundsX)?,
                self.y,
            ),
            Direction::Left => (
                self.x.checked_sub(1).ok_or(TilePosError::OutOfBoundsX)?,
                self.y,
            ),
            Direction::Down => (
                self.x,
                self.y.checked_add(1).ok_or(TilePosError::OutOfBoundsY)?,
            ),
        };

        Self::new(new_x, new_y)
    }

    #[must_use]
    pub const fn x(self) -> usize {
        self.x
    }
    #[must_use]
    pub const fn y(self) -> usize {
        self.y
    }
}
