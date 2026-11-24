use super::input::GameInput;
use crate::game::ui::context_menu::{Choice, ContextMenu, MenuContext, Outcome};
use crate::game::ui::dialog_window::{DialogButton, DialogWindow};
use crate::game::ui::UiState;
use alloc::string::String;
use alloc::vec;
use game_core::{
    plan::{Plan, PlanAction, PlanError},
    tile::TilePos,
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
    session.ui_state.tick_notice(dt);

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

    // The dialog and the context menu block the rest of the UI until they are closed
    if session.ui_state.dialog_window().is_some() {
        update_dialog(session, input);
        return;
    }

    if session.ui_state.context_menu().is_some() {
        update_context_menu(session, input);
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

    // Context menu of the tile under the cursor. By now nothing is on top of the field: a
    // dialog or a menu would have returned above
    if input.confirm && session.ui_state.is_panel_collapsed() {
        open_context_menu(session);
    }
}

fn open_context_menu(session: &mut Session) {
    let context = MenuContext::new(session, session.ui_state.cursor_pos());

    if let Some(menu) = ContextMenu::open(&context) {
        session.ui_state.open_context_menu(menu);
    } else {
        session.ui_state.show_notice("No actions available here");
    }
}

fn update_context_menu(session: &mut Session, input: GameInput) {
    if input.cancel {
        session.ui_state.back_in_menu();
        return;
    }

    if !input.confirm {
        return;
    }

    // What the chosen line leads to, read before the menu is touched
    let Some(menu) = session.ui_state.context_menu() else {
        return;
    };
    let chosen = menu.selected_line().map(|line| (line.outcome, menu.tile()));

    match chosen {
        Some((Outcome::Next(_), _)) => session.ui_state.enter_menu_level(),
        Some((Outcome::Choose(choice), tile)) => {
            session.ui_state.close_context_menu();
            choose(session, tile, choice);
        }
        _ => {}
    }
}

fn choose(session: &mut Session, tile: TilePos, choice: Choice) {
    let action = match choice {
        Choice::Build(kind) => PlanAction::Build {
            kind,
            position: tile,
        },
        Choice::Repair => PlanAction::Repair { position: tile },
        // Aiming comes together with the shots
        Choice::Aim => return,
    };

    if let Err(reason) = session.world.add_to_plan(&mut session.plan, action) {
        session.ui_state.show_notice(explain(reason));
    }
}

/// What the player gets to see when a command doesn't make it into the plan
fn explain(error: PlanError) -> &'static str {
    match error {
        PlanError::Full => "No actions left in this turn",
        PlanError::TileAlreadyBuilt => "This tile is already being built on",
        PlanError::TileAlreadyRepaired => "This tile is already being repaired",
        PlanError::GunAlreadyFiring => "This gun is already firing",
        PlanError::NotBuildable => "This can't be built",
        PlanError::NoSuchGun => "There is no gun to fire",
        PlanError::ForeignGun => "This gun belongs to the other player",
        PlanError::TileOccupied => "The tile is occupied",
        PlanError::OutsideBuildZone => "The tile is outside the building zone",
        PlanError::NothingToRepair => "There is nothing to repair here",
        PlanError::NotEnoughResources => "Not enough resources",
        // These only come from editing the plan in the side panel
        PlanError::EmptySlot
        | PlanError::PhaseMismatch
        | PlanError::NotAShot
        | PlanError::EdgeOfShootingPhase => "The plan can't be changed like this",
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

    // The next player starts with an empty plan and the cursor on the field
    session.plan = Plan::new();
    if !session.ui_state.is_panel_collapsed() {
        session.ui_state.toggle_panel();
    }

    session.world.finish_turn();
}

fn close_dialog(session: &mut Session) {
    session.ui_state.close_dialog();
}
