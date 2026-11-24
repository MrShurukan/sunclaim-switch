use super::input::GameInput;
use crate::game::ui::dialog_window::{DialogButton, DialogWindow};
use crate::game::ui::UiState;
use alloc::string::String;
use alloc::vec;
use game_core::{
    plan::Plan,
    world::{TurnPhase, World},
};

/// Major divisions of the game state. Fitting additions would be something like `MainMenu`,
/// `Settings`, `GameOver`, etc.
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
    /// Plan by the active player
    pub plan: Plan,
    pub ui_state: UiState,
    pub elapsed_time: f32,
}

impl Session {
    pub fn new() -> Self {
        Self {
            state: GameState::Running,
            world: World::new(),
            ui_state: UiState::new(),
            plan: Plan::new(),
            elapsed_time: 0.0,
        }
    }
}

/// Update (runs every frame)
pub fn update(session: &mut Session, input: GameInput, dt: f32) {
    session.elapsed_time += dt;

    match session.state {
        GameState::Running => {
            update_running(session, input, dt);
        }
        GameState::Error(_) => {}
    }
}

fn update_running(session: &mut Session, input: GameInput, dt: f32) {
    match session.world.turn_phase() {
        TurnPhase::Plan => update_plan(session, input, dt),
    }
}

fn update_plan(session: &mut Session, input: GameInput, dt: f32) {
    // Cursor move
    session.ui_state.direction_held(input.direction_raw, dt);

    // The dialog blocks the rest of the UI until it is resolved
    if session.ui_state.dialog_window().is_some() {
        update_dialog(session, input);
        return;
    }

    // Panel toggle
    if input.panel_toggle {
        session.ui_state.toggle_panel();
    }

    // Finish move
    if input.finish_turn {
        session.ui_state.open_dialog(finish_turn_dialog());
    }
}

fn update_dialog(session: &mut Session, input: GameInput) {
    if input.cancel {
        close_dialog(session);
        return;
    }

    // Presses the button the cursor is on
    if input.confirm {
        if let Some(press) = session.ui_state.selected_dialog_action() {
            press(session);
        }
    }
}

fn finish_turn_dialog() -> DialogWindow {
    DialogWindow::new(
        "Are you ready to finish your turn?",
        vec![
            DialogButton::new("Yes", finish_turn),
            DialogButton::new("No", close_dialog),
        ],
    )
}

fn finish_turn(session: &mut Session) {
    session.ui_state.close_dialog();
    session.world.finish_turn();
}

fn close_dialog(session: &mut Session) {
    session.ui_state.close_dialog();
}
