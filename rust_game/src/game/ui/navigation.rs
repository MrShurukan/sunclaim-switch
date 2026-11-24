//! Ways to move a cursor with the directions: the timing of a direction that is held down
//! and a selection moving along a list. Neither knows what is being moved.

use game_core::direction::Direction;

/// Turns a direction that is held down into single moves: one right on the press, another
/// one after a pause, and then at a steady pace
#[derive(Debug)]
pub struct DirectionRepeater {
    held: Option<HeldDirection>,
    /// The direction was already down when the moves last changed their target
    waiting_for_release: bool,
}

/// The direction held down right now and the time left until its next move
#[derive(Debug, Copy, Clone)]
struct HeldDirection {
    direction: Direction,
    pause_left: f32,
}

impl HeldDirection {
    const FIRST_PRESS_DELAY: f32 = 0.5;
    const FOLLOWING_PRESS_DELAY: f32 = 0.1;

    fn on_press(direction: Direction) -> Self {
        Self {
            direction,
            pause_left: Self::FIRST_PRESS_DELAY,
        }
    }

    fn on_move(direction: Direction) -> Self {
        Self {
            direction,
            pause_left: Self::FOLLOWING_PRESS_DELAY,
        }
    }
}

impl DirectionRepeater {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            held: None,
            waiting_for_release: false,
        }
    }

    /// Feeds the direction held down this frame, `None` if no direction is. Returns the
    /// direction every time a move has to happen
    pub fn update(&mut self, held: Option<Direction>, dt: f32) -> Option<Direction> {
        let Some(direction) = held else {
            self.held = None;
            self.waiting_for_release = false;
            return None;
        };

        // The direction was down before the moves changed their target: it isn't meant for it
        if self.waiting_for_release {
            return None;
        }

        match &mut self.held {
            // The press moves right away
            None => {
                self.held = Some(HeldDirection::on_press(direction));
                Some(direction)
            }
            Some(info) => {
                // Another direction pressed while holding counts as a new press
                if info.direction != direction {
                    self.held = Some(HeldDirection::on_press(direction));
                    return Some(direction);
                }

                info.pause_left -= dt;
                if info.pause_left < 0.0 {
                    self.held = Some(HeldDirection::on_move(direction));
                    return Some(direction);
                }

                None
            }
        }
    }

    /// Makes the repeater ignore the direction that is down right now, until it is released.
    /// Call when the moves start going to another place: the field, the panel, a dialog.
    /// A direction pressed afterwards moves right away
    pub fn interrupt(&mut self) {
        self.waiting_for_release = self.held.is_some();
        self.held = None;
    }
}

/// The way the items of a list are laid out
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// The selected item of a list laid out along one line
#[derive(Debug, Clone, Copy)]
pub struct Selection {
    index: usize,
}

impl Selection {
    /// Selects the first item to begin with
    #[must_use]
    pub const fn new() -> Self {
        Self { index: 0 }
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    /// Moves to the neighbour along the list. A direction across the list, or past its last
    /// item, leaves the selection where it is
    pub fn move_in(&mut self, direction: Direction, orientation: Orientation, item_count: usize) {
        let step = match (orientation, direction) {
            (Orientation::Horizontal, Direction::Right)
            | (Orientation::Vertical, Direction::Down) => 1,
            (Orientation::Horizontal, Direction::Left) | (Orientation::Vertical, Direction::Up) => {
                -1
            }
            _ => return,
        };

        self.index = match self.index.checked_add_signed(step) {
            Some(neighbour) => neighbour.min(item_count.saturating_sub(1)),
            None => self.index,
        };
    }
}
