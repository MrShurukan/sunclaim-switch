use crate::building::{
    Building, BuildingKind, MINER_BLUE_START, MINER_RED_START, REACTOR_BLUE_START,
    REACTOR_RED_START,
};
use crate::player::{Player, PlayerSide};
use crate::tile::{Tile, TilePos, TILES_ARRAY_SIZE, TILES_ARRAY_X_DIM};
use alloc::vec;
use alloc::vec::Vec;

pub type TileArray = [Tile; TILES_ARRAY_SIZE];

/// Platform-independent game data and rules. Rendering and input live in `rust_game`.
pub struct World {
    tiles: TileArray,

    round_number: u32,
    active_player_side: PlayerSide,
    turn_phase: TurnPhase,
    players: [Player; 2],
    buildings: Vec<Building>,
}

#[derive(Debug)]
pub enum TurnPhase {
    Plan,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new() -> Self {
        let mut world = Self {
            tiles: core::array::from_fn(|index| {
                let x = index % TILES_ARRAY_X_DIM;
                let y = index / TILES_ARRAY_X_DIM;

                Tile::new(TilePos::new(x, y).expect("Invalid world tile constructor"))
            }),

            round_number: 1,
            active_player_side: PlayerSide::Red,
            turn_phase: TurnPhase::Plan,
            players: [Player::new(PlayerSide::Red), Player::new(PlayerSide::Blue)],
            buildings: vec![
                Building::new(PlayerSide::Red, REACTOR_RED_START, BuildingKind::Reactor),
                Building::new(PlayerSide::Red, MINER_RED_START, BuildingKind::Miner),
                Building::new(PlayerSide::Blue, REACTOR_BLUE_START, BuildingKind::Reactor),
                Building::new(PlayerSide::Blue, MINER_BLUE_START, BuildingKind::Miner),
            ],
        };

        world.begin_turn();
        world
    }

    pub fn begin_turn(&mut self) {
        self.process_player_resources();
    }

    pub fn finish_turn(&mut self) {
        // TODO: Plan should be applied here

        self.switch_players();
        self.begin_turn();
    }

    fn switch_players(&mut self) {
        self.active_player_side = self.active_player_side.other();
    }

    fn process_player_resources(&mut self) {
        let mut net_change_metal: isize = 0;
        let mut net_change_energy: isize = 0;

        self.buildings
            .iter()
            .filter(|b| b.owner() == self.active_player_side)
            .for_each(|b| {
                net_change_metal += b.kind().metal_per_turn() as isize;
                net_change_energy += b.kind().energy_per_turn() as isize;
            });

        let current_player = self.active_player_mut();

        current_player.add_metal(net_change_metal);
        current_player.add_energy(net_change_energy);
    }

    #[must_use]
    pub const fn round_number(&self) -> u32 {
        self.round_number
    }

    #[must_use]
    pub const fn active_player_side(&self) -> PlayerSide {
        self.active_player_side
    }

    #[must_use]
    pub const fn active_player(&self) -> &Player {
        &self.players[self.active_player_side.index()]
    }

    #[must_use]
    fn active_player_mut(&mut self) -> &mut Player {
        self.players
            .get_mut(self.active_player_side.index())
            .expect("Correctly set up array")
    }

    #[must_use]
    pub const fn turn_phase(&self) -> &TurnPhase {
        &self.turn_phase
    }

    #[must_use]
    pub fn buildings(&self) -> &[Building] {
        &self.buildings
    }

    #[must_use]
    pub fn locate_building(&self, pos: TilePos) -> Option<&Building> {
        self.buildings.iter().find(|b| b.position() == pos)
    }
}
