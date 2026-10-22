use crate::player::PlayerSide;
use crate::tile::TilePos;
use core::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone)]
pub struct Building {
    id: BuildingId,
    owner: PlayerSide,
    position: TilePos,
    hp: u32,
    kind: BuildingKind,
}

macro_rules! unwrap_const {
    ($opt:expr) => {
        match $opt {
            Ok(x) => x,
            Err(_) => unreachable!()
        }
    };
}

pub const REACTOR_RED_START: TilePos = unwrap_const!(TilePos::new(3, 6));
pub const REACTOR_BLUE_START: TilePos = unwrap_const!(TilePos::new(20, 6));
pub const MINER_RED_START: TilePos = unwrap_const!(TilePos::new(3, 7));
pub const MINER_BLUE_START: TilePos = unwrap_const!(TilePos::new(20, 7));

impl Building {
    pub fn new(owner: PlayerSide, position: TilePos, kind: BuildingKind) -> Self {
        Self {
            id: BuildingId::next(),
            owner,
            position,
            hp: kind.start_hp(),
            kind,
        }
    }

    pub const fn id(&self) -> BuildingId {
        self.id
    }

    pub const fn owner(&self) -> PlayerSide {
        self.owner
    }

    pub const fn position(&self) -> TilePos {
        self.position
    }

    pub const fn hp(&self) -> u32 {
        self.hp
    }

    pub const fn kind(&self) -> BuildingKind {
        self.kind
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BuildingKind {
    Reactor,
    Miner
}

impl BuildingKind {
    pub const fn start_hp(self) -> u32 {
        match self {
            BuildingKind::Reactor => 100,
            BuildingKind::Miner => 35
        }
    }
}

static LAST_BUILDING_ID: AtomicU32 = AtomicU32::new(0);
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildingId(u32);

impl BuildingId {
    fn next() -> Self {
        let new_id = LAST_BUILDING_ID.fetch_add(1, Ordering::Relaxed);
        Self(new_id)
    }
}