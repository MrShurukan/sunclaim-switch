use alloc::{string::String, vec::Vec};

use super::layout::{center_horizontally, center_vertically, grown, RowCursor};
use super::layout_metrics::LAYOUT;
use super::navigation::{Orientation, Selection};
use super::theme;
use crate::game::controller::Session;
use crate::raylib::{Frame, Rect};
use game_core::direction::Direction;

/// Width of the dialog window
const WINDOW_WIDTH: f32 = 640.0;
/// Height of a button
const BUTTON_HEIGHT: f32 = 44.0;
/// Horizontal padding inside a button
const BUTTON_PADDING: f32 = 24.0;
/// Gap between the buttons
const BUTTON_GAP: f32 = 24.0;
/// The window fits its title, the row of buttons and the padding around them
const WINDOW_HEIGHT: f32 = theme::PADDING_WINDOW
    + theme::LINE_HEIGHT_TITLE
    + theme::PADDING_WINDOW
    + BUTTON_HEIGHT
    + theme::PADDING_WINDOW;

/// What a button of a dialog does when it's pressed
pub type DialogAction = fn(&mut Session);

/// Struct that holds data about current dialog window.
/// Has dynamic buttons with a callback and a text on the top.
#[derive(Debug)]
pub struct DialogWindow {
    title: String,
    buttons: Vec<DialogButton>,
    /// The button the cursor is on
    selected: Selection,
}

#[derive(Debug)]
pub struct DialogButton {
    text: String,
    action: DialogAction,
}

impl DialogWindow {
    /// The first button is selected to begin with
    #[must_use]
    pub fn new(title: impl Into<String>, buttons: Vec<DialogButton>) -> Self {
        Self {
            title: title.into(),
            buttons,
            selected: Selection::new(),
        }
    }

    /// Moves the cursor between the buttons. Directions other than left and right don't move it
    pub fn move_selection(&mut self, direction: Direction) {
        self.selected
            .move_in(direction, Orientation::Horizontal, self.buttons.len());
    }

    #[must_use]
    pub fn selected_action(&self) -> Option<DialogAction> {
        self.buttons
            .get(self.selected.index())
            .map(|button| button.action)
    }
}

impl DialogButton {
    #[must_use]
    pub fn new(text: impl Into<String>, action: DialogAction) -> Self {
        Self {
            text: text.into(),
            action,
        }
    }
}

pub fn draw(dialog: &DialogWindow, frame: &mut Frame) {
    // The game behind the dialog is dimmed
    frame.rect(LAYOUT.screen, theme::COLOR_OVERLAY);

    let window = Rect::new(
        center_horizontally(LAYOUT.screen, WINDOW_WIDTH),
        center_vertically(LAYOUT.screen, WINDOW_HEIGHT),
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    );

    // A rectangle slightly bigger than the window serves as its border
    frame.rect(grown(window, theme::WINDOW_BORDER), theme::COLOR_TEXT);
    frame.rect(window, theme::COLOR_DIALOG_BG);

    draw_title(dialog, window, frame);
    draw_buttons(dialog, window, frame);
}

#[allow(clippy::cast_precision_loss)]
fn draw_title(dialog: &DialogWindow, window: Rect, frame: &mut Frame) {
    let title_width = frame.measure_text(&dialog.title, theme::FONT_SIZE_TITLE);

    frame.text(
        &dialog.title,
        (
            center_horizontally(window, title_width as f32),
            window.y + theme::PADDING_WINDOW,
        )
            .into(),
        theme::FONT_SIZE_TITLE,
        theme::COLOR_TEXT,
    );
}

#[allow(clippy::cast_precision_loss)]
fn draw_buttons(dialog: &DialogWindow, window: Rect, frame: &mut Frame) {
    let buttons = &dialog.buttons;

    let total_width = buttons
        .iter()
        .map(|button| button_width(frame, button))
        .sum::<f32>()
        + BUTTON_GAP * buttons.len().saturating_sub(1) as f32;

    let buttons_y =
        window.y + theme::PADDING_WINDOW + theme::LINE_HEIGHT_TITLE + theme::PADDING_WINDOW;
    let mut row = RowCursor::new(
        center_horizontally(window, total_width),
        buttons_y,
        BUTTON_GAP,
    );

    for (index, button) in buttons.iter().enumerate() {
        let width = button_width(frame, button);
        let position = row.next(width);
        let button_rect = Rect::new(position.x, position.y, width, BUTTON_HEIGHT);
        let is_selected = index == dialog.selected.index();

        frame.rect(grown(button_rect, theme::WINDOW_BORDER), theme::COLOR_TEXT);
        if is_selected {
            frame.rect(button_rect, theme::COLOR_SELECTION);
        } else {
            frame.rect(button_rect, theme::COLOR_DIALOG_BG);
        }

        // Dark text reads better on the selection
        let color = if is_selected {
            theme::COLOR_PANEL_BG
        } else {
            theme::COLOR_TEXT
        };
        frame.text(
            &button.text,
            (
                button_rect.x + BUTTON_PADDING,
                center_vertically(button_rect, theme::LINE_HEIGHT_TITLE),
            )
                .into(),
            theme::FONT_SIZE_TITLE,
            color,
        );
    }
}

#[allow(clippy::cast_precision_loss)]
fn button_width(frame: &Frame, button: &DialogButton) -> f32 {
    frame.measure_text(&button.text, theme::FONT_SIZE_TITLE) as f32 + 2.0 * BUTTON_PADDING
}
