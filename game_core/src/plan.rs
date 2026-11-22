use crate::{
    building::{BuildingId, BuildingKind},
    tile::TilePos,
};

pub struct Plan {
    actions: [Option<PlanAction>; 4],
}

// TODO: Create this when walls are made
// Wall { placement: WallPlacement, position: TilePos },
pub enum PlanAction {
    /// Restrictions:
    /// 1) Can't place on non empty square
    /// 2) Can't place outside buildings bounds of the base
    /// 3) Can't place over other building ghosts in the plan
    /// 4) Cost must be acceptable (including other actions before)
    Build {
        kind: BuildingKind,
        position: TilePos,
    },
    /// Restrictions:
    /// 1) Can't use on an empty square
    /// 2) Can't repair enemy buildings
    /// 3) Can't select the same building more than twice in one plan
    /// 4) Can't select a ghost building
    /// 5) Can't repair if health is full
    /// 6) Cost must be acceptable (including other actions before)
    Repair { building_id: BuildingId },

    /// Restrictions:
    /// 1) Can't use on a ghost building
    /// 2) Can't use on a non weapon building
    /// 3) Cost must be acceptable (including other actions before)
    Shoot { building_id: BuildingId },
}
