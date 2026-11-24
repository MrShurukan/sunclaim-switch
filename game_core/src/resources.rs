#[derive(Debug, Copy, Clone, PartialEq, Eq, derive_more::Into, derive_more::Display)]
pub struct Metal(usize);
#[derive(Debug, Copy, Clone, PartialEq, Eq, derive_more::Into, derive_more::Display)]
pub struct Energy(usize);

// TODO: Remove boilerplate and duplicate code
impl Metal {
    #[must_use]
    pub const fn new(metal: usize) -> Self {
        Self(metal)
    }

    pub fn add_clamped(&mut self, net_change_metal: isize) {
        self.0 = self.0.saturating_add_signed(net_change_metal);
    }
}

impl Energy {
    #[must_use]
    pub const fn new(energy: usize) -> Self {
        Self(energy)
    }

    pub fn add_clamped(&mut self, net_change_energy: isize) {
        self.0 = self.0.saturating_add_signed(net_change_energy);
    }
}

/// Amount of metal and energy something costs or reserves
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Cost {
    pub metal: Metal,
    pub energy: Energy,
}

impl core::ops::Add for Cost {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            metal: Metal::new(self.metal.0 + rhs.metal.0),
            energy: Energy::new(self.energy.0 + rhs.energy.0),
        }
    }
}

impl Cost {
    pub const ZERO: Self = Self::new(Metal::new(0), Energy::new(0));

    #[must_use]
    pub const fn new(metal: Metal, energy: Energy) -> Self {
        Self { metal, energy }
    }

    /// Whether the budget covers both parts of the cost
    #[must_use]
    pub const fn fits_in(self, budget: Self) -> bool {
        self.metal.0 <= budget.metal.0 && self.energy.0 <= budget.energy.0
    }
}
