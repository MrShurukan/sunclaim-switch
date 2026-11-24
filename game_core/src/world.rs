use crate::building::{
    Building, BuildingId, BuildingKind, MINER_BLUE_START, MINER_RED_START, REACTOR_BLUE_START,
    REACTOR_RED_START,
};
use crate::plan::{Plan, PlanAction, PlanError};
use crate::player::{Player, PlayerSide};
use crate::resources::Cost;
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

const STARTING_PLAYER: PlayerSide = PlayerSide::Red;

/// How far the building zone reaches from a source of it, in tiles
const BUILD_ZONE_RADIUS: usize = 3;

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
            active_player_side: STARTING_PLAYER,
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
        // Increment the round counter only after two people made a move
        if self.active_player_side == STARTING_PLAYER {
            self.round_number += 1;
        }
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

    /// Whether the tile lies within the building zone of the player: a square around every
    /// reactor of theirs. Ready relays will give the same zones
    #[must_use]
    pub fn in_build_zone(&self, side: PlayerSide, position: TilePos) -> bool {
        self.buildings
            .iter()
            .filter(|source| source.owner() == side && source.kind() == BuildingKind::Reactor)
            .any(|source| within_radius(source.position(), position, BUILD_ZONE_RADIUS))
    }

    /// Whether the player can build on the tile: nothing stands on it and it lies within
    /// the player's building zone
    #[must_use]
    pub fn can_build_at(&self, side: PlayerSide, position: TilePos) -> bool {
        self.check_build_site(side, position).is_ok()
    }

    /// Whether there is something of the player's to repair on the tile: a damaged building
    /// of theirs other than the reactor. Walls on the borders of the tile will count too
    #[must_use]
    pub fn can_repair(&self, side: PlayerSide, position: TilePos) -> bool {
        self.locate_building(position).is_some_and(|building| {
            building.owner() == side
                && building.kind() != BuildingKind::Reactor
                && building.hp() < building.kind().start_hp()
        })
    }

    /// The weapon of the player standing on the tile, if there is one
    #[must_use]
    pub fn own_gun_at(&self, side: PlayerSide, position: TilePos) -> Option<&Building> {
        self.locate_building(position)
            .filter(|building| building.owner() == side && building.kind().shot_cost().is_some())
    }

    /// Adds the command to the plan after checking it against the field. From outside the
    /// core the commands get into a plan only here and through [`Self::replace_in_plan`]: the
    /// plan's own `add` and `replace` are private to the core. So every command of a plan has
    /// passed the checks of the field by the time the plan is priced or executed
    ///
    /// # Errors
    ///
    /// This function will return an error if the field doesn't allow the command: a shot needs
    /// the player's own weapon to exist on the field, a build needs a free tile within the
    /// building zone, and a repair needs something of the player's to repair. The plan together
    /// with the command has to be affordable, and the rules of the plan apply as well.
    ///
    /// ```compile_fail
    /// # hide
    /// use game_core::plan::{Plan, PlanAction};
    /// use game_core::tile::TilePos;
    ///
    /// let mut plan = Plan::new();
    /// plan.add(PlanAction::Repair {
    ///     position: TilePos::new(0, 0).unwrap(),
    /// });
    /// ```
    pub fn add_to_plan(&self, plan: &mut Plan, action: PlanAction) -> Result<usize, PlanError> {
        self.check_on_field_add(plan, action)?;

        plan.add(action)
    }

    /// Replaces the command in the slot of the plan with an edited version of it and returns
    /// the previous one. The new command goes through the same checks as in [`Self::add_to_plan`]
    ///
    /// # Errors
    ///
    /// This function will return an error for the same reasons as [`Self::add_to_plan`], and
    /// also if the slot is empty or the new command belongs to another phase
    pub fn replace_in_plan(
        &self,
        plan: &mut Plan,
        index: usize,
        action: PlanAction,
    ) -> Result<PlanAction, PlanError> {
        self.check_on_field_replace(plan, index, action)?;

        plan.replace(index, action)
    }

    /// The tile has to be free and to lie within the player's building zone
    fn check_build_site(&self, side: PlayerSide, position: TilePos) -> Result<(), PlanError> {
        if self.locate_building(position).is_some() {
            return Err(PlanError::TileOccupied);
        }

        if !self.in_build_zone(side, position) {
            return Err(PlanError::OutsideBuildZone);
        }

        Ok(())
    }

    /// The gun with this id, as long as it is on the field and is a weapon
    fn find_gun(&self, building_id: BuildingId) -> Option<&Building> {
        self.buildings
            .iter()
            .find(|building| building.id() == building_id)
            .filter(|gun| gun.kind().shot_cost().is_some())
    }

    /// The rules of the command that need the field and the stock of the active player, for
    /// adding it to the plan on top of the commands already there
    fn check_on_field_add(&self, plan: &Plan, action: PlanAction) -> Result<(), PlanError> {
        self.check_on_field(plan, None, action)
    }

    /// The rules of the command that need the field and the stock of the active player, for
    /// replacing the command of the `replaced` slot with it. The command being replaced
    /// doesn't count towards the stock: its price gives way to the price of the new one
    fn check_on_field_replace(
        &self,
        plan: &Plan,
        replaced: usize,
        action: PlanAction,
    ) -> Result<(), PlanError> {
        self.check_on_field(plan, Some(replaced), action)
    }

    /// The rules of a command that need the field and the stock of the active player. The
    /// command of the `replaced` slot doesn't count towards the stock: its price gives way to
    /// the price of the new one
    fn check_on_field(
        &self,
        plan: &Plan,
        replaced: Option<usize>,
        action: PlanAction,
    ) -> Result<(), PlanError> {
        let active_player_side = self.active_player_side();

        match action {
            PlanAction::Build { kind, position } => {
                // Whether the kind can be built at all is the plan's own rule
                if kind.cost().is_some() {
                    self.check_build_site(active_player_side, position)?;
                }
            }
            PlanAction::Repair { position } => {
                if !self.can_repair(active_player_side, position) {
                    return Err(PlanError::NothingToRepair);
                }
            }
            PlanAction::Shoot { building_id } => {
                let gun = self.find_gun(building_id).ok_or(PlanError::NoSuchGun)?;
                if gun.owner() != active_player_side {
                    return Err(PlanError::ForeignGun);
                }
            }
        }

        // The plan together with the command has to stay affordable
        if let Some(total) = self
            .price_of(action)
            .map(|price| self.plan_cost_without(plan, replaced) + price)
        {
            if !self.active_player().can_afford(total) {
                return Err(PlanError::NotEnoughResources);
            }
        }

        Ok(())
    }

    /// Total price of all commands in the plan
    ///
    /// # Panics
    ///
    /// Panics if the plan holds a command that can't be priced.
    /// Such actions should NOT appear in the plan because the logic of `add` is
    /// encapsulated (see [`Self::add_to_plan`])
    #[must_use]
    pub fn plan_cost(&self, plan: &Plan) -> Cost {
        self.plan_cost_without(plan, None)
    }

    /// Price of the plan with the command of the `left_out` slot not counted
    fn plan_cost_without(&self, plan: &Plan, left_out: Option<usize>) -> Cost {
        (0..plan.len())
            .filter(|index| Some(*index) != left_out)
            .filter_map(|index| plan.get(index))
            .fold(Cost::ZERO, |total, action| total + self.action_cost(action))
    }

    /// Price of one command, `None` when there is nothing to pay for: a shot of a gun that
    /// isn't on the field, or a kind that can't be built
    fn price_of(&self, action: PlanAction) -> Option<Cost> {
        match action {
            PlanAction::Build { kind, .. } => kind.cost(),
            PlanAction::Repair { .. } => Some(PlanAction::REPAIR_COST),
            PlanAction::Shoot { building_id } => self
                .find_gun(building_id)
                .and_then(|gun| gun.kind().shot_cost()),
        }
    }

    /// Price of one command. Expects a valid command that can be priced.
    ///
    /// # Panics
    ///
    /// Panics if the gun of a shot isn't on the field or the build names a kind that can't be built.
    fn action_cost(&self, action: PlanAction) -> Cost {
        self.price_of(action)
            .expect("a command of a valid plan can always be priced")
    }
}

/// Whether the tiles lie within the radius of each other, counted as the larger of the two
/// distances along the axes
fn within_radius(from: TilePos, to: TilePos, radius: usize) -> bool {
    from.x().abs_diff(to.x()) <= radius && from.y().abs_diff(to.y()) <= radius
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::{
        building::{
            Building, BuildingKind, MINER_BLUE_START, MINER_RED_START, REACTOR_BLUE_START,
            REACTOR_RED_START,
        },
        plan::{MoveDirection, Plan, PlanAction, PlanError},
        player::PlayerSide,
        resources::{Cost, Energy},
        tile::TilePos,
    };
    use alloc::{vec, vec::Vec};

    fn position(x: usize, y: usize) -> TilePos {
        TilePos::new(x, y).unwrap()
    }

    /// A tile at this offset from another one
    fn near(center: TilePos, dx: i32, dy: i32) -> TilePos {
        let x = i32::try_from(center.x()).unwrap() + dx;
        let y = i32::try_from(center.y()).unwrap() + dy;

        position(usize::try_from(x).unwrap(), usize::try_from(y).unwrap())
    }

    /// A tile at this offset from the reactor of Red
    fn near_reactor(dx: i32, dy: i32) -> TilePos {
        near(REACTOR_RED_START, dx, dy)
    }

    fn build(kind: BuildingKind, position: TilePos) -> PlanAction {
        PlanAction::Build { kind, position }
    }

    /// Lowers the health of the building standing on the tile
    fn damage(world: &mut World, position: TilePos, damage: u32) {
        world
            .buildings
            .iter_mut()
            .find(|building| building.position() == position)
            .unwrap()
            .take_damage(damage);
    }

    /// Brings the stock of the player to exactly this amount
    fn set_stock(world: &mut World, side: PlayerSide, stock: Cost) {
        let player = &mut world.players[side.index()];
        let to_isize = |amount: usize| isize::try_from(amount).unwrap();

        let metal: usize = player.metal().into();
        let energy: usize = player.energy().into();
        let target_metal: usize = stock.metal.into();
        let target_energy: usize = stock.energy.into();

        player.add_metal(to_isize(target_metal) - to_isize(metal));
        player.add_energy(to_isize(target_energy) - to_isize(energy));
    }

    /// Puts a weapon of Red straight onto the field and returns the command to shoot from it
    fn add_shooter(world: &mut World, kind: BuildingKind, position: TilePos) -> PlanAction {
        let building = Building::new(PlayerSide::Red, position, kind);
        let action = PlanAction::Shoot {
            building_id: building.id(),
        };

        world.buildings.push(building);

        action
    }

    /// Price of a plan holding this single command
    fn price_of(world: &World, action: PlanAction) -> Cost {
        let mut plan = Plan::new();
        world.add_to_plan(&mut plan, action).unwrap();

        world.plan_cost(&plan)
    }

    #[test]
    fn build_is_priced_as_its_kind() {
        let world = World::new();

        for kind in [
            BuildingKind::Miner,
            BuildingKind::Mortar,
            BuildingKind::DirectGun,
        ] {
            let build = build(kind, near_reactor(1, -1));

            assert_eq!(price_of(&world, build), kind.cost().unwrap());
        }
    }

    #[test]
    fn repair_has_its_own_price() {
        let mut world = World::new();
        damage(&mut world, MINER_RED_START, 1);
        let repair = PlanAction::Repair {
            position: MINER_RED_START,
        };

        assert_eq!(price_of(&world, repair), PlanAction::REPAIR_COST);
    }

    #[test]
    fn shot_is_priced_as_the_kind_of_its_gun() {
        // The command only knows the gun's id: its price comes from the field
        let mut world = World::new();
        let mortar = add_shooter(&mut world, BuildingKind::Mortar, position(5, 5));
        let direct_gun = add_shooter(&mut world, BuildingKind::DirectGun, position(6, 5));

        assert_eq!(
            price_of(&world, mortar),
            BuildingKind::Mortar.shot_cost().unwrap()
        );
        assert_eq!(
            price_of(&world, direct_gun),
            BuildingKind::DirectGun.shot_cost().unwrap()
        );
    }

    #[test]
    fn plan_price_is_the_sum_of_the_prices_of_its_commands() {
        let mut world = World::new();
        let mortar_shot = add_shooter(&mut world, BuildingKind::Mortar, position(5, 5));
        damage(&mut world, MINER_RED_START, 1);
        let commands = [
            build(BuildingKind::Miner, near_reactor(1, -1)),
            PlanAction::Repair {
                position: MINER_RED_START,
            },
            mortar_shot,
        ];

        let mut plan = Plan::new();
        assert_eq!(world.plan_cost(&plan), Cost::ZERO);

        let mut expected = Cost::ZERO;
        for action in commands {
            world.add_to_plan(&mut plan, action).unwrap();
            expected = expected + price_of(&world, action);

            assert_eq!(world.plan_cost(&plan), expected);
        }
    }

    #[test]
    fn reordered_shots_keep_the_total_price() {
        let mut world = World::new();
        let mortar = add_shooter(&mut world, BuildingKind::Mortar, position(5, 5));
        let direct_gun = add_shooter(&mut world, BuildingKind::DirectGun, position(6, 5));

        let mut plan = Plan::new();
        world.add_to_plan(&mut plan, mortar).unwrap();
        world.add_to_plan(&mut plan, direct_gun).unwrap();
        let price = world.plan_cost(&plan);

        plan.move_shot(1, MoveDirection::Up).unwrap();

        assert_eq!(plan.actions().collect::<Vec<_>>(), vec![direct_gun, mortar]);
        assert_eq!(world.plan_cost(&plan), price);
    }

    #[test]
    fn only_a_shot_of_a_weapon_on_the_field_gets_into_the_plan() {
        let mut world = World::new();
        let mortar_shot = add_shooter(&mut world, BuildingKind::Mortar, position(5, 5));
        let miner_shot = PlanAction::Shoot {
            building_id: world.locate_building(MINER_RED_START).unwrap().id(),
        };
        // A building that exists, but was never put onto the field
        let shot_of_a_missing_gun = PlanAction::Shoot {
            building_id: Building::new(PlayerSide::Red, position(6, 5), BuildingKind::DirectGun)
                .id(),
        };

        let mut plan = Plan::new();
        assert_eq!(
            world.add_to_plan(&mut plan, miner_shot),
            Err(PlanError::NoSuchGun)
        );
        assert_eq!(
            world.add_to_plan(&mut plan, shot_of_a_missing_gun),
            Err(PlanError::NoSuchGun)
        );
        assert_eq!(plan.len(), 0);

        let slot = world.add_to_plan(&mut plan, mortar_shot).unwrap();

        // An edit goes through the same checks, and the rules of the plan hold too
        assert_eq!(
            world.replace_in_plan(&mut plan, slot, miner_shot),
            Err(PlanError::NoSuchGun)
        );
        assert_eq!(
            world.add_to_plan(&mut plan, mortar_shot),
            Err(PlanError::GunAlreadyFiring)
        );
        assert_eq!(plan.get(slot), Some(mortar_shot));
        assert_eq!(
            world.plan_cost(&plan),
            BuildingKind::Mortar.shot_cost().unwrap()
        );
    }

    #[test]
    fn build_of_a_kind_without_a_price_doesnt_get_into_the_plan() {
        let world = World::new();
        let reactor = PlanAction::Build {
            kind: BuildingKind::Reactor,
            position: position(5, 5),
        };
        let mut plan = Plan::new();

        assert_eq!(
            world.add_to_plan(&mut plan, reactor),
            Err(PlanError::NotBuildable)
        );
        assert_eq!(world.plan_cost(&plan), Cost::ZERO);
    }

    #[test]
    fn build_zone_reaches_three_tiles_around_the_own_reactor() {
        let world = World::new();

        // Three tiles away along an axis and along the diagonal still count
        assert!(world.in_build_zone(PlayerSide::Red, near_reactor(3, 0)));
        assert!(world.in_build_zone(PlayerSide::Red, near_reactor(3, 3)));
        // Four tiles away don't
        assert!(!world.in_build_zone(PlayerSide::Red, near_reactor(4, 0)));
        assert!(!world.in_build_zone(PlayerSide::Red, near_reactor(3, 4)));

        // Three tiles to the left of the reactor and up from it still count, four don't
        assert!(world.in_build_zone(PlayerSide::Red, near_reactor(-3, 0)));
        assert!(world.in_build_zone(PlayerSide::Red, near_reactor(-3, -3)));
        assert!(world.in_build_zone(PlayerSide::Red, near_reactor(0, -3)));
        assert!(!world.in_build_zone(PlayerSide::Red, near_reactor(0, -4)));
        // The reactor of Red stands too close to the left edge of the map for four tiles to
        // its left to exist, so the left border shows on the reactor of Blue
        assert!(world.in_build_zone(PlayerSide::Blue, near(REACTOR_BLUE_START, -3, 0)));
        assert!(!world.in_build_zone(PlayerSide::Blue, near(REACTOR_BLUE_START, -4, 0)));

        // Every player has a zone of their own
        assert!(world.in_build_zone(PlayerSide::Blue, REACTOR_BLUE_START));
        assert!(!world.in_build_zone(PlayerSide::Blue, REACTOR_RED_START));
    }

    #[test]
    fn build_needs_a_free_tile_in_the_zone() {
        let world = World::new();
        let mortar = BuildingKind::Mortar;
        let mut plan = Plan::new();

        assert_eq!(
            world.add_to_plan(&mut plan, build(mortar, MINER_RED_START)),
            Err(PlanError::TileOccupied)
        );
        assert_eq!(
            world.add_to_plan(&mut plan, build(mortar, MINER_BLUE_START)),
            Err(PlanError::TileOccupied)
        );
        assert_eq!(
            world.add_to_plan(&mut plan, build(mortar, near_reactor(0, -4))),
            Err(PlanError::OutsideBuildZone)
        );
        assert_eq!(plan.len(), 0);

        let free = near_reactor(1, -1);
        assert!(world.can_build_at(PlayerSide::Red, free));
        // The same tile lies outside the zone of the other player
        assert!(!world.can_build_at(PlayerSide::Blue, free));
        assert!(!world.can_build_at(PlayerSide::Red, MINER_RED_START));

        assert_eq!(world.add_to_plan(&mut plan, build(mortar, free)), Ok(0));
        assert_eq!(plan.len(), 1);
    }

    #[test]
    fn plan_together_with_the_command_has_to_be_affordable() {
        let mortar = BuildingKind::Mortar.cost().unwrap();
        let two_mortars = mortar + mortar;

        let mut world = World::new();
        set_stock(&mut world, PlayerSide::Red, two_mortars);
        let mut plan = Plan::new();

        // The stock covers exactly two mortars
        world
            .add_to_plan(&mut plan, build(BuildingKind::Mortar, near_reactor(1, -1)))
            .unwrap();
        world
            .add_to_plan(&mut plan, build(BuildingKind::Mortar, near_reactor(2, -1)))
            .unwrap();
        assert_eq!(
            world.add_to_plan(&mut plan, build(BuildingKind::Mortar, near_reactor(3, -1))),
            Err(PlanError::NotEnoughResources)
        );
        assert_eq!(plan.len(), 2);

        // One unit of energy short of two mortars: the second one is already too much
        let energy: usize = two_mortars.energy.into();
        let mut short_world = World::new();
        set_stock(
            &mut short_world,
            PlayerSide::Red,
            Cost::new(two_mortars.metal, Energy::new(energy - 1)),
        );
        let mut plan = Plan::new();

        short_world
            .add_to_plan(&mut plan, build(BuildingKind::Mortar, near_reactor(1, -1)))
            .unwrap();
        assert_eq!(
            short_world.add_to_plan(&mut plan, build(BuildingKind::Mortar, near_reactor(2, -1))),
            Err(PlanError::NotEnoughResources)
        );
        assert_eq!(plan.len(), 1);
    }

    #[test]
    fn replaced_command_doesnt_count_towards_the_budget() {
        let miner = BuildingKind::Miner.cost().unwrap();
        let mut world = World::new();
        set_stock(&mut world, PlayerSide::Red, miner);
        let mut plan = Plan::new();

        // The stock covers exactly one miner, and it is already in the plan
        world
            .add_to_plan(&mut plan, build(BuildingKind::Miner, near_reactor(1, -1)))
            .unwrap();
        assert_eq!(
            world.add_to_plan(&mut plan, build(BuildingKind::Miner, near_reactor(2, -1))),
            Err(PlanError::NotEnoughResources)
        );

        // Moving it to another tile replaces its price instead of adding one
        world
            .replace_in_plan(
                &mut plan,
                0,
                build(BuildingKind::Miner, near_reactor(2, -1)),
            )
            .unwrap();
        assert_eq!(world.plan_cost(&plan), miner);
    }

    #[test]
    fn repair_needs_something_of_the_player_s_to_repair() {
        let mut world = World::new();
        let mut plan = Plan::new();

        // An intact miner has nothing to repair
        assert_eq!(
            world.add_to_plan(
                &mut plan,
                PlanAction::Repair {
                    position: MINER_RED_START
                }
            ),
            Err(PlanError::NothingToRepair)
        );

        damage(&mut world, MINER_RED_START, 1);
        damage(&mut world, REACTOR_RED_START, 1);
        damage(&mut world, MINER_BLUE_START, 1);

        assert!(world.can_repair(PlayerSide::Red, MINER_RED_START));
        // The reactor is never repaired, and the other player's buildings aren't Red's to repair
        assert!(!world.can_repair(PlayerSide::Red, REACTOR_RED_START));
        assert!(!world.can_repair(PlayerSide::Red, MINER_BLUE_START));

        assert_eq!(
            world.add_to_plan(
                &mut plan,
                PlanAction::Repair {
                    position: REACTOR_RED_START
                }
            ),
            Err(PlanError::NothingToRepair)
        );
        assert_eq!(
            world.add_to_plan(
                &mut plan,
                PlanAction::Repair {
                    position: MINER_BLUE_START
                }
            ),
            Err(PlanError::NothingToRepair)
        );
        assert_eq!(
            world.add_to_plan(
                &mut plan,
                PlanAction::Repair {
                    position: MINER_RED_START
                }
            ),
            Ok(0)
        );
    }

    #[test]
    fn shot_needs_the_player_s_own_gun() {
        let mut world = World::new();
        let own_shot = add_shooter(&mut world, BuildingKind::Mortar, position(5, 5));
        let foreign_gun = position(6, 5);
        world.buildings.push(Building::new(
            PlayerSide::Blue,
            foreign_gun,
            BuildingKind::DirectGun,
        ));
        let foreign_shot = PlanAction::Shoot {
            building_id: world.locate_building(foreign_gun).unwrap().id(),
        };

        assert!(world.own_gun_at(PlayerSide::Red, position(5, 5)).is_some());
        assert!(world.own_gun_at(PlayerSide::Red, foreign_gun).is_none());

        let mut plan = Plan::new();
        assert_eq!(
            world.add_to_plan(&mut plan, foreign_shot),
            Err(PlanError::ForeignGun)
        );
        assert_eq!(world.add_to_plan(&mut plan, own_shot), Ok(0));

        // The other way round on the turn of the other player
        world.finish_turn();
        let mut plan = Plan::new();
        assert_eq!(world.add_to_plan(&mut plan, foreign_shot), Ok(0));
        assert_eq!(
            world.add_to_plan(&mut plan, own_shot),
            Err(PlanError::ForeignGun)
        );
    }
}
