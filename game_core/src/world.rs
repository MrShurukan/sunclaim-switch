use crate::player::{Player, PlayerSide};
use crate::tile::{Tile, TilePos, TILES_ARRAY_SIZE, TILES_ARRAY_X_DIM};

pub type TileArray = [Tile; TILES_ARRAY_SIZE];

/// Platform-independent game data and rules. Rendering and input live in rust_game.
pub struct World {
    tiles: TileArray,

    round_number: u32,
    active_player: PlayerSide,
    turn_phase: TurnPhase,
    players: [Player; 2],
}

#[derive(Debug)]
pub enum TurnPhase {
    Plan,
}

impl World {
    pub fn new() -> Self {
        Self {
            tiles: core::array::from_fn(|index| {
                let x = index % TILES_ARRAY_X_DIM;
                let y = index / TILES_ARRAY_X_DIM;

                Tile::new(TilePos::new(x, y).expect("Invalid world tile constructor"))
            }),

            round_number: 1,
            active_player: PlayerSide::Red,
            turn_phase: TurnPhase::Plan,
            players: [Player::new(PlayerSide::Red), Player::new(PlayerSide::Blue)],
        }
    }

    pub const fn round_number(&self) -> u32 {
        self.round_number
    }

    pub const fn active_player_side(&self) -> PlayerSide {
        self.active_player
    }

    pub const fn active_player(&self) -> &Player {
        &self.players[self.active_player.index()]
    }

    pub const fn turn_phase(&self) -> &TurnPhase {
        &self.turn_phase
    }
}
