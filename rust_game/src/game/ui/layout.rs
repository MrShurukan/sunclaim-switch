//! Minimal layout helpers

use super::theme;
use crate::game::aseprite::{AsepriteSheet, AsepriteSheetKey};
use crate::raylib::{Color, Frame, Rect, Vec2};

/// Vertical position of content centered inside a container of `container_height`
#[must_use]
pub const fn center_vertically(container: Rect, content_height: f32) -> f32 {
    container.y + (container.height - content_height) / 2.0
}

/// Horizontal position of content centered inside a container of `content_width`
#[must_use]
pub const fn center_horizontally(container: Rect, content_width: f32) -> f32 {
    container.x + (container.width - content_width) / 2.0
}

/// Hands out positions of rows, top to bottom
pub struct ColumnCursor {
    x: f32,
    y: f32,
    gap: f32,
}

impl ColumnCursor {
    #[must_use]
    pub const fn new(x: f32, y: f32, gap: f32) -> Self {
        Self { x, y, gap }
    }

    /// Returns the position of the next row of `row_height` and advances the cursor
    pub fn next(&mut self, row_height: f32) -> Vec2 {
        let position = Vec2::new(self.x, self.y);
        self.y += row_height + self.gap;
        position
    }
}

/// Hands out positions of items, left to right
pub struct RowCursor {
    x: f32,
    y: f32,
    gap: f32,
}

impl RowCursor {
    #[must_use]
    pub const fn new(x: f32, y: f32, gap: f32) -> Self {
        Self { x, y, gap }
    }

    /// Returns the position of the next item of `item_width` and advances the cursor
    pub fn next(&mut self, item_width: f32) -> Vec2 {
        let position = Vec2::new(self.x, self.y);
        self.x += item_width + self.gap;
        position
    }
}

/// Hands out positions of items anchored to a right border, right to left.
/// Items are consumed in reverse visual order.
pub struct RowCursorFromRight {
    right_x: f32,
    y: f32,
    gap: f32,
}

impl RowCursorFromRight {
    #[must_use]
    pub const fn new(right_border: f32, y: f32, gap: f32) -> Self {
        Self {
            right_x: right_border,
            y,
            gap,
        }
    }

    /// Places the next item of `item_width` flush against the current right
    /// border and moves the border left
    pub fn next(&mut self, item_width: f32) -> Vec2 {
        self.right_x -= item_width;
        let position = Vec2::new(self.right_x, self.y);
        self.right_x -= self.gap;
        position
    }
}

/// Total width an icon group with text will occupy when drawn with
/// [`icon_group_with_text`]
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn icon_group_width<K: AsepriteSheetKey>(
    frame: &Frame,
    sheet: &AsepriteSheet<K>,
    icons: &[K],
    text: &str,
    font_size: i32,
) -> f32 {
    let mut width = 0.0;

    for (index, key) in icons.iter().enumerate() {
        width += sheet.section_size(*key).x;

        width += if index + 1 == icons.len() {
            theme::GAP_ICON_TEXT
        } else {
            theme::GAP_ICON_ICON
        };
    }

    width + frame.measure_text(text, font_size) as f32
}

/// The rectangle grown by this much on every side
#[must_use]
pub const fn grown(rect: Rect, by: f32) -> Rect {
    Rect::new(
        rect.x - by,
        rect.y - by,
        rect.width + 2.0 * by,
        rect.height + 2.0 * by,
    )
}

/// A row of a list with its lines of text going down from its top. The selected row is
/// filled with the selection color
pub fn draw_selectable_row(frame: &mut Frame, row: Rect, lines: &[&str], is_selected: bool) {
    if is_selected {
        frame.rect(row, theme::COLOR_SELECTION);
    }

    // Dark text reads better on the selection
    let color = if is_selected {
        theme::COLOR_PANEL_BG
    } else {
        theme::COLOR_TEXT
    };

    let mut text_lines =
        ColumnCursor::new(row.x + theme::PADDING_SM, row.y + theme::PADDING_SM, 0.0);

    for line in lines {
        let position = text_lines.next(theme::LINE_HEIGHT_BODY);

        frame.text(line, position, theme::FONT_SIZE_BODY, color);
    }
}

/// Draws one or several icons in a row, followed by text
pub fn icon_group_with_text<K: AsepriteSheetKey>(
    frame: &mut Frame,
    sheet: &AsepriteSheet<K>,
    icons: &[K],
    text: &str,
    position: Vec2,
    font_size: i32,
    color: Color,
) {
    let mut position = position;

    for (index, key) in icons.iter().enumerate() {
        let icon_size = sheet.draw(frame, *key, position);

        let gap = if index + 1 == icons.len() {
            theme::GAP_ICON_TEXT
        } else {
            theme::GAP_ICON_ICON
        };
        position = position.translate(icon_size.x + gap, 0.0);
    }

    frame.text(
        text,
        position.translate(0.0, theme::ICON_TEXT_DY),
        font_size,
        color,
    );
}
