//! Game UI state (focus, panels, cursor) alongside the drawing helpers it
//! relies on: the design theme, screen layout metrics and layout cursors.

use game_core::direction::Direction;
use game_core::plan::Plan;
use game_core::tile::TilePos;

use crate::game::ui::dialog_window::{DialogAction, DialogWindow};
use crate::game::ui::navigation::{DirectionRepeater, Orientation, Selection};

pub mod dialog_window;
pub(crate) mod layout;
pub(crate) mod layout_metrics;
pub(crate) mod navigation;
pub(crate) mod panel;
pub(crate) mod theme;

#[derive(Debug, Copy, Clone)]
pub enum PanelState {
    Minimized,
    Expanded,
}

impl PanelState {
    const fn other(self) -> PanelState {
        match self {
            PanelState::Minimized => PanelState::Expanded,
            PanelState::Expanded => PanelState::Minimized,
        }
    }
}

#[derive(Debug)]
pub struct UiState {
    panel_state: PanelState,
    cursor_pos: TilePos,
    /// The slot of the expanded panel the cursor is on
    selected_slot: Selection,
    direction_repeater: DirectionRepeater,
    /// Blocks other UI logic until dialog is resolved
    dialog_window: Option<DialogWindow>,
}

impl UiState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            panel_state: PanelState::Minimized,
            cursor_pos: TilePos::new(0, 0).unwrap(),
            selected_slot: Selection::new(),
            direction_repeater: DirectionRepeater::new(),
            dialog_window: None,
        }
    }

    #[must_use]
    pub const fn panel_state(&self) -> PanelState {
        self.panel_state
    }

    #[must_use]
    pub const fn cursor_pos(&self) -> TilePos {
        self.cursor_pos
    }

    #[must_use]
    pub const fn selected_slot(&self) -> usize {
        self.selected_slot.index()
    }

    #[must_use]
    pub fn dialog_window(&self) -> Option<&DialogWindow> {
        self.dialog_window.as_ref()
    }

    /// Action of the selected button of the open dialog
    #[must_use]
    pub fn selected_dialog_action(&self) -> Option<DialogAction> {
        self.dialog_window
            .as_ref()
            .and_then(DialogWindow::selected_action)
    }

    /// Moves the cursor while a direction is held down: once on the press, then repeatedly
    /// after a follow-up delay. The cursor belongs to whatever is on top: a dialog, the
    /// expanded panel, or else the field
    pub fn direction_held(&mut self, held: Option<Direction>, dt: f32) {
        if let Some(direction) = self.direction_repeater.update(held, dt) {
            self.move_cursor(direction);
        }
    }

    /// Sends the move to whatever owns the cursor right now
    fn move_cursor(&mut self, direction: Direction) {
        if let Some(dialog) = &mut self.dialog_window {
            dialog.move_selection(direction);
            return;
        }

        match self.panel_state {
            PanelState::Minimized => self.move_field_cursor(direction),
            PanelState::Expanded => {
                self.selected_slot
                    .move_in(direction, Orientation::Vertical, Plan::MAX_ACTIONS);
            }
        }
    }

    /// The cursor stays where it is at the border of the field
    fn move_field_cursor(&mut self, direction: Direction) {
        self.cursor_pos = self
            .cursor_pos
            .translate(direction)
            .unwrap_or(self.cursor_pos);
    }

    pub fn toggle_panel(&mut self) {
        self.panel_state = self.panel_state.other();

        // The direction held on the way here isn't meant for what comes next
        self.direction_repeater.interrupt();
    }

    pub fn open_dialog(&mut self, dialog: DialogWindow) {
        self.dialog_window = Some(dialog);
        self.direction_repeater.interrupt();
    }

    pub fn close_dialog(&mut self) {
        self.dialog_window = None;
        self.direction_repeater.interrupt();
    }
}
