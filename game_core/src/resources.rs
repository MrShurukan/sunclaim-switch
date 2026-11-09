#[derive(Debug, Copy, Clone, derive_more::Into, derive_more::Display)]
pub struct Metal(usize);
#[derive(Debug, Copy, Clone, derive_more::Into, derive_more::Display)]
pub struct Energy(usize);

impl Metal {
    #[must_use]
    pub const fn new(metal: usize) -> Self {
        Self(metal)
    }
}

impl Energy {
    #[must_use]
    pub const fn new(energy: usize) -> Self {
        Self(energy)
    }
}
