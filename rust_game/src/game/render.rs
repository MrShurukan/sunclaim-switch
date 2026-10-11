use alloc::format;
use game_core::tile::{TilePos, TILES_ARRAY_X_DIM, TILES_ARRAY_Y_DIM};
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::raylib::{Color, Frame};

use super::assets::Assets;
use super::controller::{ErrorInfo, GameState, Session};

#[derive(Debug)]
pub struct UiState {
    panel_visible: bool,
    cursor_pos: TilePos,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            panel_visible: true,
            cursor_pos: TilePos::new(0, 0).unwrap(),
        }
    }
}

// ====== Draw ======
pub fn draw(session: &Session, assets: &Assets, frame: &mut Frame) {
    match &session.state {
        GameState::Running => draw_running(session, assets, frame),
        GameState::Error(info) => draw_error(info, frame),
    }
}

fn draw_running(session: &Session, assets: &Assets, frame: &mut Frame) {
    // UI
    draw_top_bar(session, assets, frame);
    if session.ui_state.panel_visible {
        draw_side_panel(session, assets, frame);
    }
    draw_bottom_bar(session, assets, frame);

    // Field
    draw_field(session, assets, frame);
}

const TOP_BAR_HEIGHT: usize = 60;

fn draw_top_bar(session: &Session, assets: &Assets, frame: &mut Frame) {
    frame.rect((0, 0, SCREEN_WIDTH, TOP_BAR_HEIGHT).into(), Color::rgb(150, 150, 150));

    frame.text(format!("Round {} - {}' turn"), (15, 18).into(), 20, Color::BLACK);

    frame.const_text(c"Metal (now -> after)", (100, 10).into(), 32, Color::BLACK);
    frame.text(format!("{} -> {}"), (100, 50).into(), 32, Color::BLACK);

    frame.const_text(c"Energy", (200, 10).into(), 32, Color::BLACK);
    frame.text(format!("{} -> {}"), (200, 50).into(), 32, Color::BLACK);

    frame.const_text(c"Assigned", ())
}

fn draw_side_panel(session: &Session, assets: &Assets, frame: &mut Frame) {

}

fn draw_bottom_bar(session: &Session, assets: &Assets, frame: &mut Frame) {

}

const TILE_SIZE: usize = 40;
const FIELD_WIDTH: usize = TILE_SIZE * TILES_ARRAY_X_DIM;
const FIELD_HEIGHT: usize = TILE_SIZE * TILES_ARRAY_Y_DIM;

fn draw_field(session: &Session, assets: &Assets, frame: &mut Frame) {

}

fn draw_error(error_info: &ErrorInfo, frame: &mut Frame) {
    frame.const_text(c"Error occurred :(", (100, 20).into(), 64, Color::RED);
    frame.text(&error_info.message, (100, 100).into(), 32, Color::WHITE);
    frame.text(&error_info.origination, (100, 150).into(), 24, Color::WHITE);

    frame.const_text(c"Press ZL + Minus to reset.", (100, 400).into(), 24, Color::BLUE)
}
