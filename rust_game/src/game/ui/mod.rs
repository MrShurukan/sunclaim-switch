//! Game UI state (focus, panels, cursor) alongside the drawing helpers it
//! relies on: the design theme, screen layout metrics and layout cursors.

use game_core::direction::Direction;
use game_core::plan::Plan;
use game_core::tile::TilePos;

use crate::game::ui::context_menu::ContextMenu;
use crate::game::ui::dialog_window::{DialogAction, DialogWindow};
use crate::game::ui::navigation::{DirectionRepeater, Orientation, Selection};

pub(crate) mod context_menu;
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

/// A short message that takes the place of the hints in the bottom bar
#[derive(Debug)]
struct Notice {
    text: &'static str,
    time_left: f32,
}

/// How long a notice stays in the bottom bar
const NOTICE_SECONDS: f32 = 2.5;

#[derive(Debug)]
pub struct UiState {
    panel_state: PanelState,
    cursor_pos: TilePos,
    /// The slot of the expanded panel the cursor is on
    selected_slot: Selection,
    direction_repeater: DirectionRepeater,
    /// Blocks other UI logic until dialog is resolved
    dialog_window: Option<DialogWindow>,
    /// The menu of the tile under the cursor. Blocks the rest of the UI until it's closed
    context_menu: Option<ContextMenu>,
    notice: Option<Notice>,
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
            context_menu: None,
            notice: None,
        }
    }

    #[must_use]
    pub const fn panel_state(&self) -> PanelState {
        self.panel_state
    }

    /// Whether the side panel is collapsed
    #[must_use]
    pub const fn is_panel_collapsed(&self) -> bool {
        matches!(self.panel_state, PanelState::Minimized)
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

    #[must_use]
    pub fn context_menu(&self) -> Option<&ContextMenu> {
        self.context_menu.as_ref()
    }

    /// The message shown in the bottom bar right now, if there is one
    #[must_use]
    pub fn notice(&self) -> Option<&'static str> {
        self.notice.as_ref().map(|notice| notice.text)
    }

    /// Shows the message in the bottom bar for a few seconds
    pub fn show_notice(&mut self, text: &'static str) {
        self.notice = Some(Notice {
            text,
            time_left: NOTICE_SECONDS,
        });
    }

    /// Counts down the notice. Meant to be called every frame
    pub fn tick_notice(&mut self, dt: f32) {
        if let Some(notice) = &mut self.notice {
            notice.time_left -= dt;
            if notice.time_left <= 0.0 {
                self.notice = None;
            }
        }
    }

    /// Moves the cursor while a direction is held down: once on the press, then repeatedly
    /// after a follow-up delay. The cursor belongs to whatever is on top: a dialog, the
    /// context menu, the expanded panel, or else the field
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

        if let Some(menu) = &mut self.context_menu {
            menu.move_selection(direction);
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

    pub fn open_context_menu(&mut self, menu: ContextMenu) {
        self.context_menu = Some(menu);
        self.direction_repeater.interrupt();
    }

    pub fn close_context_menu(&mut self) {
        self.context_menu = None;
        self.direction_repeater.interrupt();
    }

    /// Enters the level the selected line of the context menu leads to
    pub fn enter_menu_level(&mut self) {
        if self.context_menu.as_mut().is_some_and(ContextMenu::enter) {
            self.direction_repeater.interrupt();
        }
    }

    /// Goes one level back in the context menu, and closes it when it already is at the root
    pub fn back_in_menu(&mut self) {
        let went_up = self.context_menu.as_mut().is_some_and(ContextMenu::back);

        if !went_up {
            self.context_menu = None;
        }

        self.direction_repeater.interrupt();
    }
}
