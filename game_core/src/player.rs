use crate::resources::{Energy, Metal};

#[derive(Debug, Copy, Clone)]
pub enum PlayerSide {
    Red,
    Blue
}

impl PlayerSide {
    pub const fn index(self) -> usize {
        match self {
            Self::Red => 0,
            Self::Blue => 1,
        }
    }

    pub const fn other(self) -> Self {
        match self {
            Self::Red => Self::Blue,
            Self::Blue => Self::Red,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Red => &"Red",
            Self::Blue => &"Blue",
        }
    }
}

pub struct Player {
    player_side: PlayerSide,
    available_energy: Energy,
    available_metal: Metal
}

impl Player {
    pub fn new(side: PlayerSide) -> Self {
        Self {
            player_side: side,
            available_energy: Energy::new(20),
            available_metal: Metal::new(30),
        }
    }

    pub fn metal(&self) -> Metal {
        self.available_metal
    }

    pub fn energy(&self) -> Energy {
        self.available_energy
    }
}