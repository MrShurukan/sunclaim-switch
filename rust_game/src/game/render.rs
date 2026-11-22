use super::assets::{Assets, Icon, TileDrawable, TileSprite};
use super::controller::{ErrorInfo, GameState, Session};
use super::ui::layout::{
    center_vertically, icon_group_width, icon_group_with_text, RowCursor, RowCursorFromRight,
};
use super::ui::layout_metrics::LAYOUT;
use super::ui::{panel, theme};
use crate::game::util::SinePulser;
use crate::raylib::{Color, Frame};
use alloc::format;
use game_core::player::PlayerSide;
use game_core::tile::{TILES_ARRAY_X_DIM, TILES_ARRAY_Y_DIM};

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
    panel::draw(session, assets, frame);
    draw_bottom_bar(assets, frame);

    // Field
    draw_field(session, assets, frame);

    draw_cursor(session, assets, frame);
}

fn draw_top_bar(session: &Session, assets: &Assets, frame: &mut Frame) {
    frame.rect(LAYOUT.top_bar, theme::COLOR_BAR_BG);

    let title_y = center_vertically(LAYOUT.top_bar.height, theme::LINE_HEIGHT_TITLE);
    frame.text(
        &format!(
            "Round {} - {}'s turn",
            session.world.round_number(),
            session.world.active_player_side().name()
        ),
        (theme::PADDING_MD, title_y).into(),
        theme::FONT_SIZE_TITLE,
        theme::COLOR_TEXT,
    );

    // TODO: The second number is the forecast after the plan is applied
    let metal = format!("{} -> {}", session.world.active_player().metal(), 0);
    let energy = format!("{} -> {}", session.world.active_player().energy(), 0);

    let items: &[(&[Icon], &str)] = &[
        (&[Icon::Metal], metal.as_str()),
        (&[Icon::Energy], energy.as_str()),
        // TODO: First number has to display current assigned plan actions
        (&[Icon::Action], "0/4 assigned"),
    ];

    // The rightmost item is consumed first, hence the reverse order
    let icons_y = center_vertically(LAYOUT.top_bar.height, theme::ROW_HEIGHT_ICON);
    let right_border = LAYOUT.screen.width - theme::PADDING_MD;
    let mut row = RowCursorFromRight::new(right_border, icons_y, theme::GAP_STATUS);

    for (icons, text) in items.iter().rev() {
        let width = icon_group_width(frame, &assets.icons, icons, text, theme::FONT_SIZE_TITLE);
        let position = row.next(width);

        icon_group_with_text(
            frame,
            &assets.icons,
            icons,
            text,
            position,
            theme::FONT_SIZE_TITLE,
            theme::COLOR_TEXT,
        );
    }
}

// TODO: Display hints based on the current UI state
const HINTS: &[(&[Icon], &str)] = &[
    (&[Icon::DPad], "Cursor"),
    (&[Icon::A], "Selection"),
    (&[Icon::LButton, Icon::RButton], "Mode"),
    (&[Icon::MinusButton], "Panel/Field"),
    (&[Icon::Y], "Finish turn"),
];

fn draw_bottom_bar(assets: &Assets, frame: &mut Frame) {
    frame.rect(LAYOUT.bottom_bar, theme::COLOR_BAR_BG);

    let y = center_vertically(LAYOUT.bottom_bar.height, theme::ROW_HEIGHT_ICON);
    let mut row = RowCursor::new(theme::PADDING_MD, y, theme::GAP_HINT);

    for (icons, text) in HINTS {
        let width = icon_group_width(frame, &assets.icons, icons, text, theme::FONT_SIZE_TITLE);
        let position = row.next(width);

        icon_group_with_text(
            frame,
            &assets.icons,
            icons,
            text,
            position,
            theme::FONT_SIZE_TITLE,
            theme::COLOR_TEXT,
        );
    }
}

fn draw_field(session: &Session, assets: &Assets, frame: &mut Frame) {
    frame.rect(LAYOUT.field, theme::COLOR_FIELD_BG);

    // Grid
    for i in 0..=TILES_ARRAY_X_DIM {
        draw_grid_line_vertical(frame, i);
    }

    for j in 0..=TILES_ARRAY_Y_DIM {
        draw_grid_line_horizontal(frame, j);
    }

    // Buildings
    for building in session.world.buildings() {
        let screen_pos = LAYOUT.tile_origin(building.position());

        assets.tiles.draw(
            frame,
            building.kind().tile_sprite(building.owner()),
            screen_pos,
        );
    }
}

fn draw_grid_line_vertical(frame: &mut Frame, x: usize) {
    let screen_x = LAYOUT.tile_x(x);

    frame.line(
        (screen_x, LAYOUT.field.y).into(),
        (screen_x, LAYOUT.field.y + LAYOUT.field.height).into(),
        1.,
        theme::COLOR_GRID_LINE,
    );
}

fn draw_grid_line_horizontal(frame: &mut Frame, y: usize) {
    let screen_y = LAYOUT.tile_y(y);

    frame.line(
        (0.0, screen_y).into(),
        (LAYOUT.field.x + LAYOUT.field.width, screen_y).into(),
        1.,
        theme::COLOR_GRID_LINE,
    );
}

const CURSOR_PULSER: SinePulser = SinePulser::new(0.5, 1.0, 5.0);

fn draw_cursor(session: &Session, assets: &Assets, frame: &mut Frame) {
    let cursor_tile = match session.world.active_player_side() {
        PlayerSide::Red => TileSprite::CursorRed,
        PlayerSide::Blue => TileSprite::CursorBlue,
    };

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    let alpha = (255.0 * libm::powf(CURSOR_PULSER.pulse(session.elapsed_time), 0.4)) as u8;
    let tint = Color::rgba(255, 255, 255, alpha);

    assets.tiles.draw_tint(
        frame,
        cursor_tile,
        LAYOUT.tile_origin(session.ui_state.cursor_pos()),
        tint,
    );
}

fn draw_error(error_info: &ErrorInfo, frame: &mut Frame) {
    frame.const_text(c"Error occurred :(", (100, 20).into(), 64, Color::RED);
    frame.text(&error_info.message, (100, 100).into(), 32, Color::WHITE);
    frame.text(&error_info.origination, (100, 150).into(), 24, Color::WHITE);

    frame.const_text(
        c"Press ZL + Minus to reset.",
        (100, 400).into(),
        24,
        Color::BLUE,
    );
}
