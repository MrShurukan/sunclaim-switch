//! The menu of the tile under the cursor. Its lines depend on the tile, and a line can lead to
//! the next level of the menu. The levels are static tables: the menu only keeps the tile and
//! the path through them, so moving between the levels creates nothing.

use super::layout::{draw_selectable_row, grown, ColumnCursor};
use super::layout_metrics::LAYOUT;
use super::navigation::{Orientation, Selection};
use super::theme;
use crate::game::controller::Session;
use crate::raylib::{Frame, Rect, Vec2};
use game_core::building::BuildingKind;
use game_core::direction::Direction;
use game_core::plan::{Plan, PlanAction};
use game_core::player::PlayerSide;
use game_core::tile::TilePos;
use game_core::world::World;

/// The most levels a menu can have
const MAX_DEPTH: usize = 4;
/// How many lines the root table has
const ROOT_LINES: usize = 3;
/// How many kinds of buildings can be built
const BUILDABLE_KINDS: usize = 3;
/// Gap between the tile and its menu
const GAP: f32 = 8.0;

/// What the lines of a menu are judged against when it opens
pub struct MenuContext<'a> {
    world: &'a World,
    plan: &'a Plan,
    side: PlayerSide,
    tile: TilePos,
}

impl<'a> MenuContext<'a> {
    /// Describes the tile for the player whose turn it is
    #[must_use]
    pub fn new(session: &'a Session, tile: TilePos) -> Self {
        Self {
            world: &session.world,
            plan: &session.plan,
            side: session.world.active_player_side(),
            tile,
        }
    }
}

/// What choosing a line of the menu does
#[derive(Debug, Clone, Copy)]
pub(crate) enum Choice {
    Build(BuildingKind),
    Repair,
    /// Switches to aiming: it comes together with the shots
    Aim,
}

/// What choosing a line of the menu leads to
#[derive(Debug, Clone, Copy)]
pub(crate) enum Outcome {
    /// The next level of the menu
    Next(&'static [MenuLine]),
    Choose(Choice),
}

/// A line of one level of the menu
#[derive(Debug)]
pub(crate) struct MenuLine {
    pub(crate) text: &'static str,
    /// Whether the line is offered for the tile the menu opens on
    offered: fn(&MenuContext) -> bool,
    pub(crate) outcome: Outcome,
}

/// The lines a tile can offer, in the order they are shown
static ROOT: [MenuLine; ROOT_LINES] = [
    MenuLine {
        text: "Build",
        offered: buildable,
        outcome: Outcome::Next(&BUILD_KINDS),
    },
    MenuLine {
        text: "Repair",
        offered: repairable,
        outcome: Outcome::Choose(Choice::Repair),
    },
    MenuLine {
        text: "Shoot",
        offered: own_gun,
        outcome: Outcome::Choose(Choice::Aim),
    },
];

/// What can be built
static BUILD_KINDS: [MenuLine; BUILDABLE_KINDS] = [
    MenuLine {
        text: BuildingKind::Miner.name(),
        offered: always,
        outcome: Outcome::Choose(Choice::Build(BuildingKind::Miner)),
    },
    MenuLine {
        text: BuildingKind::Mortar.name(),
        offered: always,
        outcome: Outcome::Choose(Choice::Build(BuildingKind::Mortar)),
    },
    MenuLine {
        text: BuildingKind::DirectGun.name(),
        offered: always,
        outcome: Outcome::Choose(Choice::Build(BuildingKind::DirectGun)),
    },
];

fn always(_: &MenuContext) -> bool {
    true
}

fn buildable(context: &MenuContext) -> bool {
    // A build already planned for the tile will get lines of its own later, like moving it
    context.world.can_build_at(context.side, context.tile)
        && !context.plan.actions().any(|action| {
            matches!(action, PlanAction::Build { position, .. } if position == context.tile)
        })
}

fn repairable(context: &MenuContext) -> bool {
    context.world.can_repair(context.side, context.tile)
}

fn own_gun(context: &MenuContext) -> bool {
    context
        .world
        .own_gun_at(context.side, context.tile)
        .is_some()
}

/// The menu of one tile. It shows one level at a time and remembers the path taken to it
#[derive(Debug)]
pub struct ContextMenu {
    /// The tile the menu was opened on
    tile: TilePos,
    /// The lines offered for the tile, in the order of the root table
    root: [Option<&'static MenuLine>; ROOT_LINES],
    /// Where the cursor was left on every level above the current one, the root first
    above: [Selection; MAX_DEPTH],
    /// How many levels were entered
    depth: usize,
    /// The line the cursor is on at the current level
    selected: Selection,
}

impl ContextMenu {
    /// Opens the menu for the tile of the context. `None` when no line is offered for it
    #[must_use]
    pub fn open(context: &MenuContext) -> Option<Self> {
        let mut menu = Self {
            tile: context.tile,
            root: [None; ROOT_LINES],
            above: [Selection::new(); MAX_DEPTH],
            depth: 0,
            selected: Selection::new(),
        };

        // The lines are picked once: nothing changes under an open menu
        let mut offered = 0;

        for line in &ROOT {
            if (line.offered)(context) {
                menu.root[offered] = Some(line);
                offered += 1;
            }
        }

        (offered > 0).then_some(menu)
    }

    /// The tile the menu was opened on
    #[must_use]
    pub const fn tile(&self) -> TilePos {
        self.tile
    }

    /// Index of the line the cursor is on
    #[must_use]
    pub const fn selected_index(&self) -> usize {
        self.selected.index()
    }

    /// The table the current level shows: the lines picked for the tile, or the ones the
    /// lines chosen on the way lead to
    fn table(&self) -> Table {
        let mut table = Table::Root;

        for chosen in &self.above[..self.depth] {
            let Outcome::Next(lines) = table.line(self, chosen.index()).outcome else {
                break;
            };

            table = Table::Next(lines);
        }

        table
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        match self.table() {
            Table::Root => self.root.iter().flatten().count(),
            Table::Next(lines) => lines.len(),
        }
    }

    /// The line with this index at the current level
    #[must_use]
    pub fn line(&self, index: usize) -> Option<&'static MenuLine> {
        match self.table() {
            Table::Root => self.root.get(index).copied().flatten(),
            Table::Next(lines) => lines.get(index),
        }
    }

    /// The line the cursor is on
    #[must_use]
    pub fn selected_line(&self) -> Option<&'static MenuLine> {
        self.line(self.selected.index())
    }

    /// Moves the cursor between the lines of the current level
    pub fn move_selection(&mut self, direction: Direction) {
        self.selected
            .move_in(direction, Orientation::Vertical, self.line_count());
    }

    /// Enters the level the selected line leads to. Returns whether there was one
    pub fn enter(&mut self) -> bool {
        let leads_deeper = matches!(
            self.selected_line().map(|line| line.outcome),
            Some(Outcome::Next(_))
        );

        if !leads_deeper || self.depth == MAX_DEPTH {
            return false;
        }

        self.above[self.depth] = self.selected;
        self.depth += 1;
        self.selected = Selection::new();

        true
    }

    /// Returns to the level above and puts the cursor back on the line it was left on.
    /// Returns whether there was a level above
    pub fn back(&mut self) -> bool {
        if self.depth == 0 {
            return false;
        }

        self.depth -= 1;
        self.selected = self.above[self.depth];

        true
    }
}

/// Where the lines of a level come from
#[derive(Clone, Copy)]
enum Table {
    /// The lines picked for the tile when the menu opened
    Root,
    /// The lines a line of the level above leads to
    Next(&'static [MenuLine]),
}

impl Table {
    fn line(self, menu: &ContextMenu, index: usize) -> &'static MenuLine {
        match self {
            Table::Root => menu.root[index].expect("a line was chosen among the offered ones"),
            Table::Next(lines) => &lines[index],
        }
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn draw(menu: &ContextMenu, frame: &mut Frame) {
    let line_count = menu.line_count();

    // Every line is as wide as the longest text, with room for the padding around it
    let longest = (0..line_count)
        .filter_map(|index| menu.line(index))
        .map(|line| frame.measure_text(line.text, theme::FONT_SIZE_BODY))
        .max()
        .unwrap_or_default() as f32;
    let line_width = longest + 2.0 * theme::PADDING_SM;

    let lines_height = line_count as f32 * theme::MENU_ROW_HEIGHT
        + line_count.saturating_sub(1) as f32 * theme::MENU_ROW_GAP;

    let window = place(
        LAYOUT.tile_origin(menu.tile),
        line_width + 2.0 * theme::MENU_PADDING,
        lines_height + 2.0 * theme::MENU_PADDING,
    );

    // A rectangle slightly bigger than the window serves as its border
    frame.rect(grown(window, theme::WINDOW_BORDER), theme::COLOR_TEXT);
    frame.rect(window, theme::COLOR_DIALOG_BG);

    let mut rows = ColumnCursor::new(
        window.x + theme::MENU_PADDING,
        window.y + theme::MENU_PADDING,
        theme::MENU_ROW_GAP,
    );

    for index in 0..line_count {
        let position = rows.next(theme::MENU_ROW_HEIGHT);
        let line = Rect::new(position.x, position.y, line_width, theme::MENU_ROW_HEIGHT);

        // A line looks like a button of a dialog: a border with a fill inside
        frame.rect(grown(line, theme::WINDOW_BORDER), theme::COLOR_TEXT);
        frame.rect(line, theme::COLOR_DIALOG_BG);
        draw_selectable_row(
            frame,
            line,
            &[menu.line(index).map_or("", |line| line.text)],
            index == menu.selected_index(),
        );
    }
}

/// Puts the menu to the right of the tile, on its height. When it doesn't fit there, it goes
/// to the left of the tile, and when it hangs below the field, it moves up just enough to fit
fn place(tile: Vec2, width: f32, height: f32) -> Rect {
    let field = LAYOUT.field;
    let tile_size = LAYOUT.tile_size_f32();

    let to_the_right = tile.x + tile_size + GAP;
    let x = if to_the_right + width <= field.x + field.width {
        to_the_right
    } else {
        tile.x - GAP - width
    };

    let field_bottom = field.y + field.height;
    let y = tile.y.min(field_bottom - height).max(field.y);

    Rect::new(x, y, width, height)
}
