use crate::resources::{Energy, Metal};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PlayerSide {
    Red,
    Blue,
}

impl PlayerSide {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Red => 0,
            Self::Blue => 1,
        }
    }

    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Red => Self::Blue,
            Self::Blue => Self::Red,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Red => "Red",
            Self::Blue => "Blue",
        }
    }
}

pub struct Player {
    side: PlayerSide,
    available_energy: Energy,
    available_metal: Metal,
}

impl Player {
    #[must_use]
    pub fn new(side: PlayerSide) -> Self {
        Self {
            side,
            available_energy: Energy::new(20),
            available_metal: Metal::new(30),
        }
    }

    #[must_use]
    pub fn metal(&self) -> Metal {
        self.available_metal
    }

    #[must_use]
    pub fn energy(&self) -> Energy {
        self.available_energy
    }

    pub fn add_metal(&mut self, net_change_metal: isize) {
        self.available_metal.add_clamped(net_change_metal);
    }

    pub fn add_energy(&mut self, net_change_energy: isize) {
        self.available_energy.add_clamped(net_change_energy);
    }
}
