use super::input::GameInput;
use crate::game::render::UiState;
use alloc::string::String;
use game_core::world::{TurnPhase, World};

/// Major divisions of the game state. Fitting additions would be something like MainMenu,
/// Settings, GameOver, etc.
#[derive(Debug)]
pub enum GameState {
    Running,
    /// Recoverable error. Similar to panic, but you can still process inputs so you can try to
    /// recover (for example restart the game)
    Error(ErrorInfo),
}

#[derive(Debug)]
pub struct ErrorInfo {
    pub message: String,
    pub origination: String,
}

/// Info belonging to the current game session. Assets live separately and survive a reset.
pub struct Session {
    pub state: GameState,
    pub world: World,
    pub ui_state: UiState,
    pub elapsed_time: f32,
}

impl Session {
    pub fn new() -> Self {
        Self {
            state: GameState::Running,
            world: World::new(),
            ui_state: UiState::new(),
            elapsed_time: 0.0,
        }
    }
}

/// Update (runs every frame)
pub fn update(session: &mut Session, input: &GameInput, dt: f32) {
    session.elapsed_time += dt;

    match session.state {
        GameState::Running => {
            update_running(session, input, dt);
        },
        GameState::Error(_) => {},
    }
}

fn update_running(session: &mut Session, input: &GameInput, dt: f32) {
    match session.world.turn_phase() {
        TurnPhase::Plan => update_plan(session, input, dt),
    }
}

fn update_plan(session: &mut Session, input: &GameInput, dt: f32) {
    // Cursor move
    session.ui_state.direction_held(input.direction_raw, dt);
}
