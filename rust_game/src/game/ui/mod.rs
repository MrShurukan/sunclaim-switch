//! Game UI state (focus, panels, cursor) alongside the drawing helpers it
//! relies on: the design theme, screen layout metrics and layout cursors.

use game_core::direction::Direction;
use game_core::tile::{TilePos, TilePosError};

pub(crate) mod layout;
pub(crate) mod layout_metrics;
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
    direction_hold_info: Option<DirectionHoldInfo>,
}

#[derive(Debug, Copy, Clone)]
struct DirectionHoldInfo {
    direction: Direction,
    hold_pause_left: f32,
}

impl DirectionHoldInfo {
    const FIRST_PRESS_DELAY: f32 = 0.5;
    const FOLLOWING_PRESS_DELAY: f32 = 0.1;

    fn new_first(direction: Direction) -> Self {
        Self {
            direction,
            hold_pause_left: Self::FIRST_PRESS_DELAY,
        }
    }

    fn new_following(direction: Direction) -> Self {
        Self {
            direction,
            hold_pause_left: Self::FOLLOWING_PRESS_DELAY,
        }
    }
}

#[derive(Debug, Copy, Clone, derive_more::From)]
pub enum CursorMoveError {
    InvalidCursorPosition(TilePosError),
    PanelIsOpen,
}

impl UiState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            panel_state: PanelState::Minimized,
            cursor_pos: TilePos::new(0, 0).unwrap(),
            direction_hold_info: None,
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

    fn try_translate_cursor(&mut self, dir: Direction) -> Result<(), CursorMoveError> {
        if !matches!(self.panel_state, PanelState::Minimized) {
            return Err(CursorMoveError::PanelIsOpen);
        }

        self.cursor_pos = self.cursor_pos.translate(dir)?;

        Ok(())
    }

    /// Moves the cursor while a direction is held down: once on the press,
    /// then repeatedly after a follow-up delay
    pub fn direction_held(&mut self, dir: Option<Direction>, dt: f32) {
        // Button released
        if dir.is_none() {
            self.direction_hold_info = None;
            return;
        }

        let dir = dir.unwrap();

        match &mut self.direction_hold_info {
            // First press, we advance the cursor (if possible) and remember the button was pressed
            None => self.new_direction_pressed(dir),

            // Following presses
            Some(info) => {
                // Did we change the direction?
                if info.direction != dir {
                    self.new_direction_pressed(dir);
                    return;
                }

                // Otherwise proceed to calculate time
                info.hold_pause_left -= dt;
                if info.hold_pause_left < 0.0 {
                    let _ = self.try_translate_cursor(dir);
                    self.direction_hold_info = Some(DirectionHoldInfo::new_following(dir));
                }
            }
        }
    }

    #[inline]
    fn new_direction_pressed(&mut self, dir: Direction) {
        let _ = self.try_translate_cursor(dir);

        self.direction_hold_info = Some(DirectionHoldInfo::new_first(dir));
    }

    pub fn toggle_panel(&mut self) {
        self.panel_state = self.panel_state.other();
    }
}
