use crate::{
    building::{BuildingId, BuildingKind},
    resources::{Cost, Energy, Metal},
    tile::TilePos,
};

/// Commands planned by one player for their turn. The commands are kept in the order they are
/// executed in: all builds, then all repairs, then all shots, with the empty slots at the end.
/// So a slot index means the same for the execution and for the plan list in the UI.
///
/// The plan keeps the rules that need the plan alone: at most [`Plan::MAX_ACTIONS`] commands,
/// a kind that can be built and no target used twice. The rules that need the field, like the
/// gun of a shot, are the world's to check. That is why the commands get into a plan from
/// outside the core only through [`crate::world::World::add_to_plan`]
pub struct Plan {
    slots: [Option<PlanAction>; Self::MAX_ACTIONS],
}

impl Plan {
    /// Maximum number of commands in a plan. Every command spends one of the turn actions
    pub const MAX_ACTIONS: usize = 4;

    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: [None; Self::MAX_ACTIONS],
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.slots.iter().flatten().count()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.get(0).is_none()
    }

    /// Command in the slot with this index. `None` if the slot is empty or doesn't exist
    #[must_use]
    pub fn get(&self, index: usize) -> Option<PlanAction> {
        self.slots.get(index).copied().flatten()
    }

    /// Commands in the execution order, without the empty slots
    pub fn actions(&self) -> impl Iterator<Item = PlanAction> + '_ {
        self.slots.iter().flatten().copied()
    }

    /// Adds the command after the last command of its phase, moving the commands of the
    /// later phases back. Returns the index of the slot the command took
    ///
    /// # Errors
    ///
    /// This function will return an error if the plan already holds [`Plan::MAX_ACTIONS`]
    /// commands, if the command can't be planned at all, like a build of the reactor, or if it
    /// conflicts with one of them: the same tile is built on or repaired twice, or the same gun
    /// shoots twice.
    pub(crate) fn add(&mut self, action: PlanAction) -> Result<usize, PlanError> {
        let len = self.len();
        if len == Self::MAX_ACTIONS {
            return Err(PlanError::Full);
        }

        if let Some(error) = is_impossible(action) {
            return Err(error);
        }

        if let Some(conflict) = self.conflict(action, None) {
            return Err(conflict);
        }

        let index = self.insert_index(action.phase());

        // Free the slot by shifting the following commands one slot to the right
        self.slots[index..=len].rotate_right(1);
        self.slots[index] = Some(action);

        Ok(index)
    }

    /// Removes the command from the slot and returns it. The commands after it move one slot forward
    pub fn remove(&mut self, index: usize) -> Option<PlanAction> {
        let removed = self.get(index)?;

        let last = self.len() - 1;
        self.slots[index..=last].rotate_left(1);
        // The rotation leaves the removed command in the last occupied slot
        self.slots[last] = None;

        Some(removed)
    }

    /// Replaces the command in the slot with an edited version of it and returns the previous one
    ///
    /// # Errors
    ///
    /// This function will return an error if the slot is empty or doesn't exist, if the new
    /// command belongs to another phase than the command it replaces, if it can't be planned at
    /// all, see [`Plan::add`], or if it conflicts with one of the other commands.
    pub(crate) fn replace(
        &mut self,
        index: usize,
        action: PlanAction,
    ) -> Result<PlanAction, PlanError> {
        let previous = self.get(index).ok_or(PlanError::EmptySlot)?;

        // A command is edited in place, so it can't turn into a command of another phase
        if previous.phase() != action.phase() {
            return Err(PlanError::PhaseMismatch);
        }

        if let Some(error) = is_impossible(action) {
            return Err(error);
        }

        if let Some(conflict) = self.conflict(action, Some(index)) {
            return Err(conflict);
        }

        self.slots[index] = Some(action);

        Ok(previous)
    }

    /// Swaps the shot in the slot with the shot next to it in the shooting order
    ///
    /// # Errors
    ///
    /// This function will return an error if the slot is empty or doesn't exist, doesn't hold
    /// a shot, or the shot already is the first or the last one.
    pub fn move_shot(&mut self, index: usize, direction: MoveDirection) -> Result<(), PlanError> {
        let Some(action) = self.get(index) else {
            return Err(PlanError::EmptySlot);
        };

        // Only shots change places: builds and repairs are fixed in their phases
        if action.phase() != ActionPhase::Shoot {
            return Err(PlanError::NotAShot);
        }

        let neighbour = match direction {
            MoveDirection::Up => index.checked_sub(1),
            MoveDirection::Down => Some(index + 1),
        };

        // A shot can only swap places with another shot
        let Some(neighbour) = neighbour.filter(|&other| self.is_shot(other)) else {
            return Err(PlanError::EdgeOfShootingPhase);
        };

        self.slots.swap(index, neighbour);

        Ok(())
    }

    /// Slot a command of this phase takes: right after the last command of its own or an
    /// earlier phase
    fn insert_index(&self, phase: ActionPhase) -> usize {
        self.slots
            .iter()
            .flatten()
            .position(|action| action.phase() > phase)
            .unwrap_or_else(|| self.len())
    }

    fn is_shot(&self, index: usize) -> bool {
        self.get(index)
            .is_some_and(|action| action.phase() == ActionPhase::Shoot)
    }

    /// Why the command can't join the plan because of the plan alone. The command of the
    /// `replaced` slot is left out: the new command takes its place
    fn conflict(&self, action: PlanAction, replaced: Option<usize>) -> Option<PlanError> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(index, _)| Some(*index) != replaced)
            .filter_map(|(_, other)| *other)
            .find_map(|other| conflict_between(action, other))
    }
}

/// Why the command can't be planned at all, whatever the plan and the field are
fn is_impossible(action: PlanAction) -> Option<PlanError> {
    match action {
        PlanAction::Build { kind, .. } if kind.cost().is_none() => Some(PlanError::NotBuildable),
        _ => None,
    }
}

/// Why two commands can't belong to one plan
fn conflict_between(first: PlanAction, second: PlanAction) -> Option<PlanError> {
    match (first, second) {
        (
            PlanAction::Build { position, .. },
            PlanAction::Build {
                position: other, ..
            },
        ) if position == other => Some(PlanError::TileAlreadyBuilt),
        (PlanAction::Repair { position }, PlanAction::Repair { position: other })
            if position == other =>
        {
            Some(PlanError::TileAlreadyRepaired)
        }
        (PlanAction::Shoot { building_id }, PlanAction::Shoot { building_id: other })
            if building_id == other =>
        {
            Some(PlanError::GunAlreadyFiring)
        }
        _ => None,
    }
}

impl Default for Plan {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// The plan already holds [`Plan::MAX_ACTIONS`] commands
    Full,
    /// The slot is empty or doesn't exist
    EmptySlot,
    /// The new command belongs to another phase than the command it replaces
    PhaseMismatch,
    /// The slot holds a build or a repair command, only shots change places
    NotAShot,
    /// The shot already is the first or the last one, so there is no other shot to swap it with
    EdgeOfShootingPhase,
    /// Another build of the plan already stands on this tile
    TileAlreadyBuilt,
    /// Another repair of the plan already covers this tile
    TileAlreadyRepaired,
    /// Another shot of the plan is already fired by this gun
    GunAlreadyFiring,
    /// The kind of the build can't be built: the reactor is starting only
    NotBuildable,
    /// The gun of the shot isn't on the field or isn't a weapon
    NoSuchGun,
    /// A building already stands on the tile of the build
    TileOccupied,
    /// The tile of the build lies outside the building zone of the player
    OutsideBuildZone,
    /// There is nothing of the player's to repair on the tile
    NothingToRepair,
    /// The gun of the shot belongs to the other player
    ForeignGun,
    /// The plan together with the command costs more than the player has
    NotEnoughResources,
}

/// Direction of a shot move inside the shooting phase
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveDirection {
    /// Towards the shots fired earlier
    Up,
    /// Towards the shots fired later
    Down,
}

/// Part of the turn execution a command belongs to. The variants are ordered like the phases
/// are executed
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionPhase {
    Build,
    Repair,
    Shoot,
}

// TODO: Create this when walls are made. One command builds a straight run of 1 to 3 segments,
// with every segment checked and paid for separately:
// Wall { placement: WallPlacement, position: TilePos, length: u8 },
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlanAction {
    /// Restrictions:
    /// 1) Can't place on non empty square
    /// 2) Can't place outside buildings bounds of the base (ghost relays don't extend it in the
    ///    same plan)
    /// 3) Can't place over other building ghosts in the plan
    /// 4) Can't build the reactor, it is starting only
    /// 5) Cost must be acceptable (including other actions before)
    Build {
        kind: BuildingKind,
        position: TilePos,
    },
    /// Restrictions:
    /// 1) At least one own damaged object has to be in or around the tile
    /// 2) Can't repair enemy objects
    /// 3) Can't select the same tile more than once in one plan
    /// 4) Ghost buildings are not repaired
    /// 5) Can't repair the reactor, though the walls around it can be repaired
    /// 6) Cost must be acceptable (including other actions before)
    ///
    /// Side note: repairs the own damaged building in the tile together with the own damaged
    /// walls on its four edges, so an empty tile with damaged walls is a valid target too
    Repair { position: TilePos },

    /// Restrictions:
    /// 1) Can't use on a ghost building
    /// 2) Can't use on a non weapon building
    /// 3) Can't shoot twice from the same gun in one plan
    /// 4) Cost must be acceptable (including other actions before)
    Shoot { building_id: BuildingId },
}

impl PlanAction {
    /// Price of one repair, the same however many objects it restores
    pub const REPAIR_COST: Cost = Cost::new(Metal::new(3), Energy::new(3));

    /// Part of the turn execution the command belongs to
    #[must_use]
    pub const fn phase(self) -> ActionPhase {
        match self {
            PlanAction::Build { .. } => ActionPhase::Build,
            PlanAction::Repair { .. } => ActionPhase::Repair,
            PlanAction::Shoot { .. } => ActionPhase::Shoot,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ActionPhase, MoveDirection, Plan, PlanAction, PlanError};
    use crate::{
        building::{Building, BuildingId, BuildingKind},
        player::PlayerSide,
        tile::TilePos,
        world::World,
    };
    use alloc::{vec, vec::Vec};

    /// A tile of the top row: the tests only need to tell the tiles apart
    fn tile(x: usize) -> TilePos {
        TilePos::new(x, 0).unwrap()
    }

    /// Id of a gun that doesn't have to be on the field: the plan only compares the ids
    fn any_gun_id() -> BuildingId {
        Building::new(PlayerSide::Red, tile(0), BuildingKind::DirectGun).id()
    }

    fn build(kind: BuildingKind, position: TilePos) -> PlanAction {
        PlanAction::Build { kind, position }
    }

    fn repair(position: TilePos) -> PlanAction {
        PlanAction::Repair { position }
    }

    /// A shot of a gun of its own, so that the shots don't conflict with each other
    fn shoot() -> PlanAction {
        PlanAction::Shoot {
            building_id: any_gun_id(),
        }
    }

    fn actions_of(plan: &Plan) -> Vec<PlanAction> {
        plan.actions().collect()
    }

    /// The commands go by phases and the empty slots all stay at the end
    fn assert_well_formed(plan: &Plan) {
        let phases: Vec<ActionPhase> = plan.actions().map(PlanAction::phase).collect();
        let mut sorted = phases.clone();
        sorted.sort();

        assert_eq!(phases, sorted);
        assert!(plan.slots[plan.len()..].iter().all(Option::is_none));
    }

    #[test]
    fn fifth_command_is_rejected() {
        let mut plan = Plan::new();

        for x in 0..Plan::MAX_ACTIONS {
            plan.add(build(BuildingKind::Miner, tile(x))).unwrap();
        }

        let fifth = build(BuildingKind::Miner, tile(Plan::MAX_ACTIONS));
        assert_eq!(plan.add(fifth), Err(PlanError::Full));
        assert_eq!(plan.len(), Plan::MAX_ACTIONS);
    }

    #[test]
    fn commands_land_in_phase_order_whatever_order_they_are_added_in() {
        let mut plan = Plan::new();

        plan.add(shoot()).unwrap();
        plan.add(repair(tile(1))).unwrap();
        plan.add(build(BuildingKind::Miner, tile(0))).unwrap();

        let phases: Vec<ActionPhase> = plan.actions().map(PlanAction::phase).collect();
        assert_eq!(
            phases,
            [ActionPhase::Build, ActionPhase::Repair, ActionPhase::Shoot]
        );
    }

    #[test]
    fn phase_order_survives_every_operation() {
        let mut plan = Plan::new();
        let [shot, repair, miner, mortar] = [
            shoot(),
            repair(tile(3)),
            build(BuildingKind::Miner, tile(0)),
            build(BuildingKind::Mortar, tile(1)),
        ];

        // Added against the execution order
        for action in [shot, repair, miner, mortar] {
            plan.add(action).unwrap();
        }
        assert_well_formed(&plan);
        assert_eq!(actions_of(&plan), vec![miner, mortar, repair, shot]);

        plan.remove(0).unwrap();
        assert_well_formed(&plan);
        assert_eq!(actions_of(&plan), vec![mortar, repair, shot]);

        let edited = shoot();
        plan.replace(2, edited).unwrap();
        assert_well_formed(&plan);
        assert_eq!(actions_of(&plan), vec![mortar, repair, edited]);

        plan.remove(1).unwrap();
        assert_well_formed(&plan);
        assert_eq!(actions_of(&plan), vec![mortar, edited]);

        let gun = build(BuildingKind::DirectGun, tile(2));
        plan.add(gun).unwrap();
        assert_well_formed(&plan);
        assert_eq!(actions_of(&plan), vec![mortar, gun, edited]);
    }

    #[test]
    fn removal_frees_the_price_of_the_command() {
        let world = World::new();
        let mut plan = Plan::new();
        let miner = build(BuildingKind::Miner, tile(0));
        plan.add(miner).unwrap();
        plan.add(build(BuildingKind::Mortar, tile(1))).unwrap();
        let with_miner = world.plan_cost(&plan);

        assert_eq!(plan.remove(0), Some(miner));
        assert_eq!(
            world.plan_cost(&plan) + BuildingKind::Miner.cost().unwrap(),
            with_miner
        );
    }

    #[test]
    fn new_shot_goes_to_the_end_of_the_queue() {
        let mut plan = Plan::new();
        let [first, second] = [shoot(), shoot()];
        let miner = build(BuildingKind::Miner, tile(0));

        plan.add(first).unwrap();
        plan.add(miner).unwrap();
        let index = plan.add(second).unwrap();

        assert_eq!(index, 2);
        assert_eq!(actions_of(&plan), vec![miner, first, second]);
    }

    #[test]
    fn shot_swaps_places_with_the_neighbouring_shot() {
        let mut plan = Plan::new();
        let [repair_of_tile, first, second, third] = [repair(tile(1)), shoot(), shoot(), shoot()];
        for action in [repair_of_tile, first, second, third] {
            plan.add(action).unwrap();
        }

        plan.move_shot(3, MoveDirection::Up).unwrap();
        assert_eq!(
            actions_of(&plan),
            vec![repair_of_tile, first, third, second]
        );

        plan.move_shot(2, MoveDirection::Down).unwrap();
        assert_eq!(
            actions_of(&plan),
            vec![repair_of_tile, first, second, third]
        );
    }

    #[test]
    fn builds_and_repairs_keep_their_places() {
        let mut plan = Plan::new();
        let [miner, repair_of_tile] = [build(BuildingKind::Miner, tile(0)), repair(tile(1))];
        plan.add(miner).unwrap();
        plan.add(repair_of_tile).unwrap();

        assert_eq!(
            plan.move_shot(0, MoveDirection::Up),
            Err(PlanError::NotAShot)
        );
        assert_eq!(
            plan.move_shot(0, MoveDirection::Down),
            Err(PlanError::NotAShot)
        );
        assert_eq!(
            plan.move_shot(1, MoveDirection::Up),
            Err(PlanError::NotAShot)
        );
        assert_eq!(actions_of(&plan), vec![miner, repair_of_tile]);
    }

    #[test]
    fn shot_cannot_leave_the_shooting_phase() {
        let mut plan = Plan::new();
        let [repair_of_tile, shot] = [repair(tile(1)), shoot()];
        plan.add(repair_of_tile).unwrap();
        plan.add(shot).unwrap();

        // The only shot has no other shot to swap with
        assert_eq!(
            plan.move_shot(1, MoveDirection::Up),
            Err(PlanError::EdgeOfShootingPhase)
        );
        assert_eq!(
            plan.move_shot(1, MoveDirection::Down),
            Err(PlanError::EdgeOfShootingPhase)
        );
        assert_eq!(actions_of(&plan), vec![repair_of_tile, shot]);
    }

    #[test]
    fn empty_slots_are_reported() {
        let mut plan = Plan::new();
        plan.add(shoot()).unwrap();

        assert_eq!(plan.remove(1), None);
        assert_eq!(plan.remove(Plan::MAX_ACTIONS), None);
        assert_eq!(plan.replace(1, shoot()), Err(PlanError::EmptySlot));
        assert_eq!(
            plan.move_shot(1, MoveDirection::Up),
            Err(PlanError::EmptySlot)
        );
    }

    #[test]
    fn replacement_keeps_the_slot_and_returns_the_previous_command() {
        let mut plan = Plan::new();
        let [miner, shot] = [build(BuildingKind::Miner, tile(0)), shoot()];
        plan.add(shot).unwrap();
        plan.add(miner).unwrap();

        let edited = shoot();

        assert_eq!(plan.replace(1, edited), Ok(shot));
        assert_eq!(actions_of(&plan), vec![miner, edited]);
    }

    #[test]
    fn replacement_cannot_change_the_phase() {
        let mut plan = Plan::new();
        let shot = shoot();
        plan.add(shot).unwrap();

        assert_eq!(
            plan.replace(0, repair(tile(1))),
            Err(PlanError::PhaseMismatch)
        );
        assert_eq!(actions_of(&plan), vec![shot]);
    }

    #[test]
    fn tile_can_be_built_on_once() {
        let mut plan = Plan::new();
        let miner = build(BuildingKind::Miner, tile(0));
        plan.add(miner).unwrap();

        // The same tile, another kind
        let mortar = build(BuildingKind::Mortar, tile(0));
        assert_eq!(plan.add(mortar), Err(PlanError::TileAlreadyBuilt));
        assert_eq!(actions_of(&plan), vec![miner]);

        // The same kind, another tile
        plan.add(build(BuildingKind::Miner, tile(1))).unwrap();
        assert_eq!(plan.len(), 2);
    }

    #[test]
    fn tile_can_be_repaired_once() {
        let mut plan = Plan::new();
        let first = repair(tile(0));
        plan.add(first).unwrap();

        assert_eq!(
            plan.add(repair(tile(0))),
            Err(PlanError::TileAlreadyRepaired)
        );
        assert_eq!(actions_of(&plan), vec![first]);

        plan.add(repair(tile(1))).unwrap();
        assert_eq!(plan.len(), 2);
    }

    #[test]
    fn gun_can_shoot_once() {
        let mut plan = Plan::new();
        let shot = shoot();
        plan.add(shot).unwrap();

        assert_eq!(plan.add(shot), Err(PlanError::GunAlreadyFiring));
        assert_eq!(actions_of(&plan), vec![shot]);

        // Another gun can shoot in the same plan
        plan.add(shoot()).unwrap();
        assert_eq!(plan.len(), 2);
    }

    #[test]
    fn kind_without_a_price_cant_be_built() {
        let mut plan = Plan::new();
        let reactor = build(BuildingKind::Reactor, tile(0));

        assert_eq!(plan.add(reactor), Err(PlanError::NotBuildable));
        assert_eq!(plan.len(), 0);

        // The same rule holds for an edit of an existing command
        let miner = build(BuildingKind::Miner, tile(0));
        plan.add(miner).unwrap();
        assert_eq!(plan.replace(0, reactor), Err(PlanError::NotBuildable));
        assert_eq!(actions_of(&plan), vec![miner]);
    }

    #[test]
    fn replacement_only_conflicts_with_the_other_commands() {
        let mut plan = Plan::new();
        let [first, second] = [shoot(), shoot()];
        plan.add(first).unwrap();
        plan.add(second).unwrap();

        // The gun of the other shot is already firing
        assert_eq!(plan.replace(0, second), Err(PlanError::GunAlreadyFiring));
        // Its own gun isn't, as far as this command is concerned
        assert_eq!(plan.replace(0, first), Ok(first));
        assert_eq!(actions_of(&plan), vec![first, second]);
    }
}
