//! Design tokens: colors, font sizes and spacing used across the game UI.
//! Positions of individual elements are computed by the layout cursors
//! (see [`super::layout`]) from these values.

use crate::raylib::Color;

// ===== Colors =====

/// Background of the top and bottom bars
pub const COLOR_BAR_BG: Color = Color::rgb(17, 40, 71);
/// Background of the side panel
pub const COLOR_PANEL_BG: Color = Color::rgb(12, 29, 51);
/// Background of the field
pub const COLOR_FIELD_BG: Color = Color::rgb(194, 137, 81);
/// Grid lines on the field
pub const COLOR_GRID_LINE: Color = Color::WHITE;
/// Regular UI text
pub const COLOR_TEXT: Color = Color::WHITE;

// ===== Font sizes =====

pub const FONT_SIZE_TITLE: i32 = 20;
pub const FONT_SIZE_BODY: i32 = 16;

// ===== Spacing and heights =====

/// Small padding between related elements
pub const PADDING_SM: f32 = 5.0;
/// Standard padding from the edge of a bar or panel
pub const PADDING_MD: f32 = 15.0;
/// Vertical gap between rows in a column
pub const GAP_ROW: f32 = 5.0;
/// Horizontal gap between hint groups in the bottom bar
pub const GAP_HINT: f32 = 30.0;
/// Horizontal gap between status groups in the top bar
pub const GAP_STATUS: f32 = 40.0;
/// Horizontal gap between an icon and its text
pub const GAP_ICON_TEXT: f32 = 6.0;
/// Horizontal gap between two icons inside one group
pub const GAP_ICON_ICON: f32 = 7.0;
/// Vertical offset of the text standing next to an icon
pub const ICON_TEXT_DY: f32 = 8.0;
/// Height of a text row written in [`FONT_SIZE_BODY`]
pub const LINE_HEIGHT_BODY: f32 = 22.0;
/// Height of a text row written in [`FONT_SIZE_TITLE`]
pub const LINE_HEIGHT_TITLE: f32 = 22.0;
/// Height of a row consisting of icons with text
pub const ROW_HEIGHT_ICON: f32 = 28.0;
