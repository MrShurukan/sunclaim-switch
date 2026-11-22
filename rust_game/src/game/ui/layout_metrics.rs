//! Computed geometry of the game screen: the sizes and positions of the main
//! areas (bars, field, side panel)

use super::super::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::raylib::{Rect, Vec2};
use game_core::tile::{TilePos, TILES_ARRAY_X_DIM, TILES_ARRAY_Y_DIM};

/// Layout of the whole game screen
pub const LAYOUT: LayoutMetrics = LayoutMetrics::new();

/// Size of one field tile in pixels
const TILE_SIZE: usize = 48;
/// Height of the top status bar
const TOP_BAR_HEIGHT: usize = 60;

/// Screen geometry shared by all renderers. Used in [`LAYOUT`] constant.
pub struct LayoutMetrics {
    /// Whole screen bounds
    pub screen: Rect,
    pub top_bar: Rect,
    pub bottom_bar: Rect,
    /// The playfield, positioned right under the top bar
    pub field: Rect,
    /// Collapsed side panel to the right of the field
    pub side_panel: Rect,
    pub tile_size: usize,

    /// Top-left pixel of the field, kept separately for the const tile math
    field_origin: (usize, usize),
}

impl LayoutMetrics {
    const fn new() -> Self {
        let field_origin = (0, TOP_BAR_HEIGHT);
        let field_width = TILE_SIZE * TILES_ARRAY_X_DIM;
        let field_height = TILE_SIZE * TILES_ARRAY_Y_DIM;

        Self {
            screen: Rect::new_usize(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT),
            top_bar: Rect::new_usize(0, 0, SCREEN_WIDTH, TOP_BAR_HEIGHT),
            field: Rect::new_usize(field_origin.0, field_origin.1, field_width, field_height),
            bottom_bar: Rect::new_usize(
                0,
                TOP_BAR_HEIGHT + field_height,
                SCREEN_WIDTH,
                SCREEN_HEIGHT - TOP_BAR_HEIGHT - field_height,
            ),
            side_panel: Rect::new_usize(
                field_width,
                TOP_BAR_HEIGHT,
                SCREEN_WIDTH - field_width,
                field_height,
            ),
            tile_size: TILE_SIZE,
            field_origin,
        }
    }

    /// X of the left edge of the tile column `x`.
    /// `x == TILES_ARRAY_X_DIM` gives the right border of the field (for grid lines).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub const fn tile_x(&self, x: usize) -> f32 {
        self.field_origin.0 as f32 + (x * self.tile_size) as f32
    }

    /// Y of the top edge of the tile row `y`.
    /// `y == TILES_ARRAY_Y_DIM` gives the bottom border of the field (for grid lines).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub const fn tile_y(&self, y: usize) -> f32 {
        self.field_origin.1 as f32 + (y * self.tile_size) as f32
    }

    /// Top-left corner of a tile in screen coordinates
    #[must_use]
    pub const fn tile_origin(&self, pos: TilePos) -> Vec2 {
        Vec2::new(self.tile_x(pos.x()), self.tile_y(pos.y()))
    }

    /// Tile size as a float, for mixing with other float geometry
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub const fn tile_size_f32(&self) -> f32 {
        self.tile_size as f32
    }
}
