use crate::player::PlayerSide;
use crate::resources::{Cost, Energy, Metal};
use crate::tile::TilePos;
use core::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub struct Building {
    id: BuildingId,
    owner: PlayerSide,
    position: TilePos,
    hp: u32,
    state: BuildingState,
}

macro_rules! unwrap_const {
    ($opt:expr) => {
        match $opt {
            Ok(x) => x,
            Err(_) => unreachable!(),
        }
    };
}

pub const REACTOR_RED_START: TilePos = unwrap_const!(TilePos::new(3, 6));
pub const REACTOR_BLUE_START: TilePos = unwrap_const!(TilePos::new(20, 6));
pub const MINER_RED_START: TilePos = unwrap_const!(TilePos::new(3, 7));
pub const MINER_BLUE_START: TilePos = unwrap_const!(TilePos::new(20, 7));

impl Building {
    #[must_use]
    pub fn new(owner: PlayerSide, position: TilePos, kind: BuildingKind) -> Self {
        Self {
            id: BuildingId::next(),
            owner,
            position,
            hp: kind.start_hp(),
            state: kind.into(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> BuildingId {
        self.id
    }

    #[must_use]
    pub const fn owner(&self) -> PlayerSide {
        self.owner
    }

    #[must_use]
    pub const fn position(&self) -> TilePos {
        self.position
    }

    #[must_use]
    pub const fn hp(&self) -> u32 {
        self.hp
    }

    #[must_use]
    pub const fn kind(&self) -> BuildingKind {
        self.state.kind()
    }
}

#[derive(Debug)]
pub enum BuildingState {
    Reactor,
    Miner,
    Mortar {
        angle: f32,
        power: usize,
        last_shot_position: Option<(f32, f32)>,
    },
    DirectGun {
        angle: f32,
        last_shot_position: Option<(f32, f32)>,
    },
}
impl BuildingState {
    const fn kind(&self) -> BuildingKind {
        match self {
            BuildingState::Reactor => BuildingKind::Reactor,
            BuildingState::Miner => BuildingKind::Miner,
            BuildingState::Mortar { .. } => BuildingKind::Mortar,
            BuildingState::DirectGun { .. } => BuildingKind::DirectGun,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuildingKind {
    Reactor,
    Miner,
    Mortar,
    DirectGun,
}

impl From<BuildingKind> for BuildingState {
    fn from(value: BuildingKind) -> Self {
        match value {
            BuildingKind::Reactor => BuildingState::Reactor,
            BuildingKind::Miner => BuildingState::Miner,
            BuildingKind::Mortar => BuildingState::Mortar {
                angle: 0.0,
                power: 0,
                last_shot_position: None,
            },
            BuildingKind::DirectGun => BuildingState::DirectGun {
                angle: 0.0,
                last_shot_position: None,
            },
        }
    }
}

impl BuildingKind {
    /// Price to build the kind. `None` for the kinds that can't be built
    #[must_use]
    pub const fn cost(self) -> Option<Cost> {
        match self {
            // The reactor is starting only
            BuildingKind::Reactor => None,
            BuildingKind::Miner => Some(Cost::new(Metal::new(12), Energy::new(4))),
            BuildingKind::Mortar => Some(Cost::new(Metal::new(18), Energy::new(10))),
            BuildingKind::DirectGun => Some(Cost::new(Metal::new(14), Energy::new(8))),
        }
    }

    /// Price of one shot of the kind. `None` for the kinds that can't shoot
    #[must_use]
    pub const fn shot_cost(self) -> Option<Cost> {
        match self {
            BuildingKind::Reactor | BuildingKind::Miner => None,
            BuildingKind::Mortar => Some(Cost::new(Metal::new(1), Energy::new(4))),
            BuildingKind::DirectGun => Some(Cost::new(Metal::new(0), Energy::new(3))),
        }
    }

    #[must_use]
    pub const fn start_hp(self) -> u32 {
        match self {
            BuildingKind::Reactor => 100,
            BuildingKind::Miner => 35,
            BuildingKind::Mortar => 40,
            BuildingKind::DirectGun => 50,
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            BuildingKind::Reactor => "Reactor",
            BuildingKind::Miner => "Miner",
            BuildingKind::Mortar => "Mortar",
            BuildingKind::DirectGun => "Gun",
        }
    }

    #[must_use]
    pub fn metal_per_turn(self) -> u8 {
        match self {
            BuildingKind::Reactor | BuildingKind::Miner => 4,
            BuildingKind::Mortar | BuildingKind::DirectGun => 0,
        }
    }

    #[must_use]
    pub fn energy_per_turn(self) -> u8 {
        match self {
            BuildingKind::Reactor => 12,
            BuildingKind::Miner | BuildingKind::Mortar | BuildingKind::DirectGun => 0,
        }
    }
}

// TODO: Attach to the world so each new world starts with fresh ids
static LAST_BUILDING_ID: AtomicU32 = AtomicU32::new(0);
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildingId(u32);

impl BuildingId {
    fn next() -> Self {
        let new_id = LAST_BUILDING_ID.fetch_add(1, Ordering::Relaxed);
        Self(new_id)
    }
}

#[cfg(test)]
mod tests {
    use super::BuildingKind;

    #[test]
    fn reactor_can_be_neither_bought_nor_fired() {
        assert_eq!(BuildingKind::Reactor.cost(), None);
        assert_eq!(BuildingKind::Reactor.shot_cost(), None);
        assert_eq!(BuildingKind::Miner.shot_cost(), None);
    }
}
