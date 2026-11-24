use crate::building::{
    Building, BuildingKind, MINER_BLUE_START, MINER_RED_START, REACTOR_BLUE_START,
    REACTOR_RED_START,
};
use crate::plan::{Plan, PlanAction};
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

    /// Total price of all commands in the plan
    ///
    /// # Panics
    ///
    /// Panics if the plan holds a command that can't be priced: a shot without its gun on the
    /// field, or a build of a kind that can't be built.
    #[must_use]
    pub fn plan_cost(&self, plan: &Plan) -> Cost {
        plan.actions()
            .fold(Cost::ZERO, |total, action| total + self.action_cost(action))
    }

    /// Price of one command. Expects a valid command that can be priced.
    ///
    /// # Panics
    ///
    /// Panics if the gun of a shot isn't on the field or the build names a kind that can't be built.
    fn action_cost(&self, action: PlanAction) -> Cost {
        let price = match action {
            PlanAction::Build { kind, .. } => kind.cost(),
            PlanAction::Repair { .. } => Some(PlanAction::REPAIR_COST),
            PlanAction::Shoot { building_id } => self
                .buildings
                .iter()
                .find(|building| building.id() == building_id)
                .and_then(|gun| gun.kind().shot_cost()),
        };

        price.expect("a command of a valid plan can always be priced")
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::{
        building::{Building, BuildingKind, MINER_RED_START},
        plan::{MoveDirection, Plan, PlanAction},
        player::PlayerSide,
        resources::Cost,
        tile::TilePos,
    };
    use alloc::{vec, vec::Vec};

    fn position(x: usize, y: usize) -> TilePos {
        TilePos::new(x, y).unwrap()
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
        plan.add(action).unwrap();

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
            let build = PlanAction::Build {
                kind,
                position: position(0, 0),
            };

            assert_eq!(price_of(&world, build), kind.cost().unwrap());
        }
    }

    #[test]
    fn repair_is_priced_the_same_whatever_the_tile_holds() {
        let world = World::new();
        let empty_tile = PlanAction::Repair {
            position: position(0, 0),
        };
        let tile_of_the_miner = PlanAction::Repair {
            position: MINER_RED_START,
        };

        assert_eq!(price_of(&world, empty_tile), PlanAction::REPAIR_COST);
        assert_eq!(price_of(&world, tile_of_the_miner), PlanAction::REPAIR_COST);
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
        let commands = [
            PlanAction::Build {
                kind: BuildingKind::Miner,
                position: position(0, 0),
            },
            PlanAction::Repair {
                position: position(1, 0),
            },
            mortar_shot,
        ];

        let mut plan = Plan::new();
        assert_eq!(world.plan_cost(&plan), Cost::ZERO);

        let mut expected = Cost::ZERO;
        for action in commands {
            plan.add(action).unwrap();
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
        plan.add(mortar).unwrap();
        plan.add(direct_gun).unwrap();
        let price = world.plan_cost(&plan);

        plan.move_shot(1, MoveDirection::Up).unwrap();

        assert_eq!(plan.actions().collect::<Vec<_>>(), vec![direct_gun, mortar]);
        assert_eq!(world.plan_cost(&plan), price);
    }
}
