use super::assets::{Assets, Icons, TileSprites};
use super::controller::{ErrorInfo, GameState, Session};
use crate::game::aseprite::{AsepriteSheet, AsepriteSheetKey};
use crate::game::util::SinePulser;
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::raylib::{Color, Frame, Vec2};
use alloc::format;
use game_core::player::PlayerSide;
use game_core::tile::{TilePos, TILES_ARRAY_X_DIM, TILES_ARRAY_Y_DIM};

#[derive(Debug, Copy, Clone)]
pub enum PanelState {
    Minimized,
    Expanded
}

#[derive(Debug)]
pub struct UiState {
    panel_state: PanelState,
    cursor_pos: TilePos,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            panel_state: PanelState::Minimized,
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
    draw_side_panel(session, assets, frame);
    draw_bottom_bar(session, assets, frame);

    // Field
    draw_field(session, assets, frame);

    draw_cursor(session, assets, frame);
}

const TOP_BAR_HEIGHT: usize = 60;
const BAR_COLOR: Color = Color::rgb(17, 40, 71);

fn draw_top_bar(session: &Session, assets: &Assets, frame: &mut Frame) {
    frame.rect((0, 0, SCREEN_WIDTH, TOP_BAR_HEIGHT).into(), BAR_COLOR);

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
        "0/4 assigned",
        (1092, 14).into(),
        Color::WHITE
    );
}

const SIDE_PANEL_START_X: usize = FIELD_WIDTH;
const SIDE_PANEL_START_Y: usize = TOP_BAR_HEIGHT;
const SIDE_PANEL_HEIGHT: usize = BOTTOM_BAR_START_Y - SIDE_PANEL_START_Y;
const SIDE_PANEL_WIDTH: usize = SCREEN_WIDTH - SIDE_PANEL_START_X;

const SIDE_PANEL_COLOR: Color = Color::rgb(12, 29, 51);

fn draw_side_panel(session: &Session, assets: &Assets, frame: &mut Frame) {
    // TODO: Expanded panel
    frame.rect(
        (SIDE_PANEL_START_X, SIDE_PANEL_START_Y, SIDE_PANEL_WIDTH, SIDE_PANEL_HEIGHT).into(),
        SIDE_PANEL_COLOR
    );

    // TODO: Sidepanel info
}

const BOTTOM_BAR_START_Y: usize = TOP_BAR_HEIGHT + FIELD_HEIGHT;
const BOTTOM_BAR_HEIGHT: usize = SCREEN_HEIGHT - BOTTOM_BAR_START_Y;

fn draw_bottom_bar(session: &Session, assets: &Assets, frame: &mut Frame) {
    frame.rect((0, BOTTOM_BAR_START_Y, SCREEN_WIDTH, BOTTOM_BAR_HEIGHT).into(), BAR_COLOR);

    // TODO: Display help based on the current UI state
    // TODO: Make it a streamlined experience showing predefined help from an array and calculating coords automatically
    const Y: usize = BOTTOM_BAR_START_Y + 26;

    icon_with_text(frame, &assets.icons, Icons::DPad, "Cursor", (15, Y).into(), Color::WHITE);
    icon_with_text(frame, &assets.icons, Icons::A, "Selection", (144, Y).into(), Color::WHITE);
    icons_with_text(frame, &assets.icons,
                    &[Icons::LButton, Icons::RButton], "Mode", (297, Y).into(), Color::WHITE);
    icon_with_text(frame, &assets.icons, Icons::MinusButton, "Panel/Field", (445, Y).into(), Color::WHITE);
    icon_with_text(frame, &assets.icons, Icons::Y, "Finish turn", (615, Y).into(), Color::WHITE);
}

const TILE_SIZE: usize = 48;
const FIELD_Y_START: usize = TOP_BAR_HEIGHT;
const FIELD_WIDTH: usize = TILE_SIZE * TILES_ARRAY_X_DIM;
const FIELD_HEIGHT: usize = TILE_SIZE * TILES_ARRAY_Y_DIM;

fn draw_field(session: &Session, assets: &Assets, frame: &mut Frame) {
    frame.rect((0, 60, FIELD_WIDTH, FIELD_HEIGHT).into(), Color::rgb(194, 137, 81));

    // Grid
    for i in 0..=TILES_ARRAY_X_DIM {
        vertical_line(frame, i * TILE_SIZE);

        for j in 0..=TILES_ARRAY_Y_DIM {
            horizontal_line(frame, j * TILE_SIZE + TOP_BAR_HEIGHT);
        }
    }
}

#[inline]
fn vertical_line(frame: &mut Frame, x: usize) {
    frame.line((x, FIELD_Y_START).into(), (x, FIELD_HEIGHT + TOP_BAR_HEIGHT).into(), 1., Color::WHITE);
}

#[inline]
fn horizontal_line(frame: &mut Frame, y: usize) {
    frame.line((0, y).into(), (FIELD_WIDTH, y).into(), 1., Color::WHITE);
}

const CURSOR_PULSER: SinePulser = SinePulser::new(0.5, 1.0, 5.0);

fn draw_cursor(session: &Session, assets: &Assets, frame: &mut Frame) {
    let cursor_tile = match session.world.active_player_side() {
        PlayerSide::Red => TileSprites::CursorRed,
        PlayerSide::Blue => TileSprites::CursorBlue,
    };

    let position = session.ui_state.cursor_pos;

    let alpha = (255.0 * libm::powf(CURSOR_PULSER.pulse(session.elapsed_time), 0.4)) as u8;
    let tint = Color::rgba(255, 255, 255, alpha);

    assets.tiles.draw_tint(
        frame,
        cursor_tile,
        (position.x() * TILE_SIZE, FIELD_Y_START + position.y() * TILE_SIZE).into(),
        tint
    );
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

fn icons_with_text<K: AsepriteSheetKey>(
    frame: &mut Frame,
    sheet: &AsepriteSheet<K>,
    icon_keys: &[K],
    text: &str,
    mut position: Vec2,
    color: Color
) {
    for key in icon_keys {
        let icon_size = sheet.draw(frame, *key, position);
        position = position.translate(icon_size.x + 7.0, 0.0);
    }

    position = position.translate(0.0, 8.0);
    frame.text(text, position, 20, color);
}