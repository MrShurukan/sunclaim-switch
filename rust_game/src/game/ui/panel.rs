//! The side panel: a collapsed card of the selected tile, or the expanded
//! plan overview (once implemented).

use super::layout::{center_vertically, icon_group_with_text, ColumnCursor};
use super::layout_metrics::LAYOUT;
use super::{theme, PanelState};
use crate::game::assets::{Assets, Icon, TileDrawable};
use crate::game::controller::Session;
use crate::raylib::{Color, Frame, Rect, Vec2};
use alloc::format;
use game_core::plan::Plan;

/// Offset of the preview icon from the corner of the panel
const CARD_ICON_OFFSET: Vec2 = Vec2::new(5.0, 6.0);
/// Vertical offset of the title text from the top of the preview icon
const CARD_TITLE_DY: f32 = 20.0;
/// Gap between the bottom of the preview icon and the first card row
const CARD_CONTENT_TOP_GAP: f32 = 23.0;

pub fn draw(session: &Session, assets: &Assets, frame: &mut Frame) {
    match session.ui_state.panel_state() {
        PanelState::Minimized => draw_minimized(session, assets, frame),
        PanelState::Expanded => draw_expanded(session, assets, frame),
    }
}

/// Height of one slot row of the expanded panel
const SLOT_HEIGHT: f32 = 40.0;
/// Gap between the slot rows
const SLOT_GAP: f32 = 8.0;

/// TODO: The expanded panel slides over the field and shows the full plan overview: fixed
/// build/repair rows and reorderable shots. For now it only lists the slots
fn draw_expanded(session: &Session, _assets: &Assets, frame: &mut Frame) {
    let panel = LAYOUT.side_panel;
    frame.rect(panel, theme::COLOR_PANEL_BG);

    let selected = session.ui_state.selected_slot();
    let content_x = panel.x + theme::PADDING_MD;
    let content_width = panel.width - 2.0 * theme::PADDING_MD;
    let mut rows = ColumnCursor::new(content_x, panel.y + theme::PADDING_MD, SLOT_GAP);

    for slot in 0..Plan::MAX_ACTIONS {
        let position = rows.next(SLOT_HEIGHT);
        let row = Rect::new(position.x, position.y, content_width, SLOT_HEIGHT);
        let is_selected = slot == selected;

        if is_selected {
            frame.rect(row, theme::COLOR_SELECTION);
        }

        // Dark text reads better on the selection
        let color = if is_selected {
            theme::COLOR_PANEL_BG
        } else {
            theme::COLOR_TEXT
        };
        frame.text(
            &format!("{}. {}", slot + 1, slot_label(session, slot)),
            (
                position.x + theme::PADDING_SM,
                center_vertically(row, theme::LINE_HEIGHT_BODY),
            )
                .into(),
            theme::FONT_SIZE_BODY,
            color,
        );
    }
}

/// Commands have no names of their own yet
fn slot_label(session: &Session, slot: usize) -> &'static str {
    if session.plan.get(slot).is_some() {
        "Command"
    } else {
        "Empty"
    }
}

fn draw_minimized(session: &Session, assets: &Assets, frame: &mut Frame) {
    let panel = LAYOUT.side_panel;
    frame.rect(panel, theme::COLOR_PANEL_BG);

    // Preview icon background
    let icon_pos = Vec2::new(panel.x, panel.y) + CARD_ICON_OFFSET;
    frame.rect(
        (
            icon_pos.x,
            icon_pos.y,
            LAYOUT.tile_size_f32(),
            LAYOUT.tile_size_f32(),
        )
            .into(),
        Color::WHITE,
    );

    let Some(building) = session.world.locate_building(session.ui_state.cursor_pos()) else {
        draw_card_title(frame, icon_pos, "Empty");
        return;
    };

    let sprite = building.kind().tile_sprite(building.owner());
    assets.tiles.draw(frame, sprite, icon_pos);

    draw_card_title(frame, icon_pos, building.kind().name());

    let content_x = icon_pos.x + theme::PADDING_SM;
    let content_y = icon_pos.y + LAYOUT.tile_size_f32() + CARD_CONTENT_TOP_GAP;
    let mut rows = ColumnCursor::new(content_x, content_y, theme::GAP_ROW);

    // Health
    let hp_pos = rows.next(theme::LINE_HEIGHT_BODY);
    frame.text(
        &format!("{}/{} HP", building.hp(), building.kind().start_hp()),
        hp_pos,
        theme::FONT_SIZE_BODY,
        theme::COLOR_TEXT,
    );

    // Resources per turn
    let per_turn_pos = rows.next(theme::LINE_HEIGHT_BODY);
    frame.const_text(
        c"Per turn:",
        per_turn_pos,
        theme::FONT_SIZE_BODY,
        theme::COLOR_TEXT,
    );

    let metal_pos = rows.next(theme::ROW_HEIGHT_ICON);
    icon_group_with_text(
        frame,
        &assets.icons,
        &[Icon::Metal],
        &format!("+{}", building.kind().metal_per_turn()),
        metal_pos,
        theme::FONT_SIZE_TITLE,
        theme::COLOR_TEXT,
    );

    let energy_pos = rows.next(theme::ROW_HEIGHT_ICON);
    icon_group_with_text(
        frame,
        &assets.icons,
        &[Icon::Energy],
        &format!("+{}", building.kind().energy_per_turn()),
        energy_pos,
        theme::FONT_SIZE_TITLE,
        theme::COLOR_TEXT,
    );
}

fn draw_card_title(frame: &mut Frame<'_>, icon_pos: Vec2, title: &str) {
    let title_pos = icon_pos + Vec2::new(LAYOUT.tile_size_f32() + theme::PADDING_SM, CARD_TITLE_DY);

    frame.text(title, title_pos, theme::FONT_SIZE_BODY, theme::COLOR_TEXT);
}
