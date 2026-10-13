use super::assets::{Assets, Icons};
use super::controller::{ErrorInfo, GameState, Session};
use crate::game::aseprite::{AsepriteSheet, AsepriteSheetKey};
use crate::game::SCREEN_WIDTH;
use crate::raylib::{Color, Frame, Vec2};
use alloc::format;
use game_core::tile::{TilePos, TILES_ARRAY_X_DIM, TILES_ARRAY_Y_DIM};

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
    frame.rect((0, 0, SCREEN_WIDTH, TOP_BAR_HEIGHT).into(), Color::rgb(17, 40, 71));

    frame.text(
        &format!("Round {} - {}'s turn", session.world.round_number(), session.world.active_player_side().name()),
        (15, 21).into(),
        20,
        Color::WHITE
    );

    icon_with_text(
        frame,
        &assets.icons,
        Icons::Metal,
        &format!("{} -> {}", session.world.active_player().metal(), 0),
        (680, 14).into(),
        Color::WHITE
    );

    icon_with_text(
        frame,
        &assets.icons,
        Icons::Energy,
        &format!("{} -> {}", session.world.active_player().energy(), 0),
        (886, 14).into(),
        Color::WHITE
    );

    icon_with_text(
        frame,
        &assets.icons,
        Icons::Action,
        &"0/4 assigned",
        (1092, 14).into(),
        Color::WHITE
    );
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

fn icon_with_text<K: AsepriteSheetKey>(
    frame: &mut Frame,
    sheet: &AsepriteSheet<K>,
    icon_key: K,
    text: &str,
    position: Vec2,
    color: Color
) {
    let icon_size = sheet.draw(frame, icon_key, position);

    let position = position.translate(icon_size.x + 6.0, 8.0);
    frame.text(text, position, 20, color);
}