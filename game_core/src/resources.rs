#[derive(Debug, Copy, Clone, derive_more::Into, derive_more::Display)]
pub struct Metal(usize);
#[derive(Debug, Copy, Clone, derive_more::Into, derive_more::Display)]
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
