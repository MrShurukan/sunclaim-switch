use crate::raylib::{Button, Gamepad};
use game_core::direction::Direction;

#[derive(Debug, Clone, Copy, Default)]
pub struct GameInput {
    pub reset: bool,
    /// Only has Some on the very frame the button was pressed
    pub direction_pressed: Option<Direction>,
    /// Has Some if direction is held down
    pub direction_raw: Option<Direction>,
}

trait GamepadRead {
    fn from_gamepad(gamepad: &Gamepad) -> Option<Self> where Self: Sized;
    fn from_gamepad_raw(gamepad: &Gamepad) -> Option<Self> where Self: Sized;
}
impl GamepadRead for Direction {
    fn from_gamepad(gamepad: &Gamepad) -> Option<Direction> {
        if gamepad.pressed(Button::Up) {
            Some(Direction::Up)
        } else if gamepad.pressed(Button::Right) {
            Some(Direction::Right)
        } else if gamepad.pressed(Button::Left) {
            Some(Direction::Left)
        } else if gamepad.pressed(Button::Down) {
            Some(Direction::Down)
        } else {
            None
        }
    }

    fn from_gamepad_raw(gamepad: &Gamepad) -> Option<Direction> {
        if gamepad.down(Button::Up) {
            Some(Direction::Up)
        } else if gamepad.down(Button::Right) {
            Some(Direction::Right)
        } else if gamepad.down(Button::Left) {
            Some(Direction::Left)
        } else if gamepad.down(Button::Down) {
            Some(Direction::Down)
        } else {
            None
        }
    }
}

impl GameInput {
    pub fn read(gamepad: &Gamepad) -> Self {
        Self {
            reset:
                (gamepad.pressed(Button::Minus) && gamepad.down(Button::ZL)) ||
                (gamepad.pressed(Button::ZL)    && gamepad.down(Button::Minus)),

            direction_pressed: Direction::from_gamepad(gamepad),
            direction_raw: Direction::from_gamepad_raw(gamepad),
        }
    }
}
