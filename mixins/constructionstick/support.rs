// Game-side glue for Construction Sticks slice 1 (plane construction).
// Injected into `crate::game` as `constructionstick_state`; it needs the game,
// so it is not part of the standalone library (only `runtime.rs` is).
//
// The clicked block is the template: cells grow in the clicked face's plane
// wherever the block behind them is the same as the clicked block. Each cell
// consumes one item of that block's kind (the stick is the tool) and wears the
// held stick by one, with a rollback when the supply runs out.
//
// A cell is placed through vanilla's own placement (`Behaviours::place_block`),
// so it gets the block's `getStateForPlacement` (a stair keeps its facing), its
// survival and entity-obstruction checks, `setPlacedBy` and the place sound.
// Only a placement that lands in the planned cell is accepted: a cell vanilla
// would replace or place beside is skipped, never placed out of the plane.

use horizon_core::geometry::{Direction, Vec3};
use horizon_core::position::BlockPos;
use horizon_world::level::flags;
use horizon_world::level::{PlaceContext, PlaceOutcome};
use horizon_world::registry::BlockStateId;
use winit::keyboard::KeyCode;

use crate::game::constructionstick::{accepts, plan, stick_for, World};

/// The item that places `block`, from the block's own data (a block without an
/// item, e.g. fire, returns None: it cannot be supplied).
fn item_of(game: &crate::game::Game<'_>, block: &str) -> Option<String> {
    let state = game.pack.assets.registry.default_state(block)?;
    let item = &*game.res.behaviours().data(state).item;
    (!item.is_empty() && item != "minecraft:air").then(|| item.to_owned())
}

/// The state to place for a block name (used for the air rollback).
fn state_of(game: &crate::game::Game<'_>, name: &str) -> Option<BlockStateId> {
    game.pack.assets.registry.default_state(name)
}

/// The name of the block at `pos`, if the level has it loaded.
fn block_name(game: &crate::game::Game<'_>, [x, y, z]: [i32; 3]) -> Option<String> {
    let world = &game.world;
    if !(-30_000_000..30_000_000).contains(&x)
        || !(-30_000_000..30_000_000).contains(&z)
        || !(world.shape.min_y..world.shape.max_y()).contains(&y)
        || !world.loaded(x, z)
    {
        return None;
    }
    game.pack
        .assets
        .registry
        .state(world.block(x, y, z))
        .map(|state| state.name.clone())
}

fn direction(face: [i32; 3]) -> Direction {
    match face {
        [0, 1, 0] => Direction::Up,
        [0, -1, 0] => Direction::Down,
        [0, 0, -1] => Direction::North,
        [0, 0, 1] => Direction::South,
        [-1, 0, 0] => Direction::West,
        _ => Direction::East,
    }
}

/// The vanilla `BlockPlaceContext` for placing `item` into `cell`, clicked on
/// `face` of `clicked`: the hit point is the centre of the clicked face, so a
/// slab still has a plausible vertical hit.
fn context(
    game: &crate::game::Game<'_>,
    cell: [i32; 3],
    clicked: [i32; 3],
    face: [i32; 3],
    item: &str,
) -> PlaceContext {
    let [nx, ny, nz] = face;
    PlaceContext {
        clicked_pos: BlockPos::new(cell[0], cell[1], cell[2]),
        face: direction(face),
        hit: Vec3::new(
            f64::from(clicked[0]) + 0.5 + f64::from(nx) * 0.5,
            f64::from(clicked[1]) + 0.5 + f64::from(ny) * 0.5,
            f64::from(clicked[2]) + 0.5 + f64::from(nz) * 0.5,
        ),
        inside: false,
        yaw: game.yaw,
        pitch: game.pitch,
        sneaking: game.keys.contains(&KeyCode::ShiftLeft),
        item: item.to_owned(),
        can_use_game_master_blocks: game.can_use_game_master_blocks(),
    }
}

struct GameWorld<'a, 'g> {
    game: &'a mut crate::game::Game<'g>,
    clicked: [i32; 3],
    face: [i32; 3],
    item: String,
}

impl World for GameWorld<'_, '_> {
    fn block(&mut self, pos: [i32; 3]) -> Option<String> {
        block_name(self.game, pos)
    }

    fn can_place(&mut self, pos: [i32; 3]) -> bool {
        let ctx = context(&*self.game, pos, self.clicked, self.face, &self.item);
        let resolved = self
            .game
            .with_level(|b, level, ticks| b.placement_state(level, ticks, &ctx));
        let Some((at, _)) = resolved else {
            return false;
        };
        accepts(pos, Some([at.x, at.y, at.z]))
    }
}

/// A right click with a construction stick on a block: fill the clicked face's
/// plane with copies of the clicked block, claiming the use.
#[allow(private_interfaces)]
pub fn on_use_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    let Some(tier) = stick_for(&stack.item) else {
        return;
    };
    let Some(player) = game.native_acting_player() else {
        return;
    };
    let dimension = player.dimension.clone();
    let Some(clicked) = block_name(game, pos) else {
        return;
    };
    let Some(item) = item_of(game, &clicked) else {
        return;
    };
    let supply = game.native_count_items(&item);
    let limit = if game.creative {
        tier.limit
    } else {
        tier.limit.min(supply)
    };
    if limit == 0 {
        return;
    }
    let cells = {
        let mut world = GameWorld {
            game,
            clicked: pos,
            face,
            item: item.clone(),
        };
        plan(&mut world, pos, face, &clicked, limit)
    };
    if cells.is_empty() {
        return;
    }
    *claimed = true;
    for cell in cells {
        let ctx = context(game, cell, pos, face, &item);
        let outcome = game.with_level(|b, level, ticks| b.place_block(level, ticks, &ctx));
        let PlaceOutcome::Placed { .. } = outcome else {
            // Vanilla refused this cell (survival, obstruction): skip it and
            // keep filling the rest of the plane, as upstream's snapshots do.
            continue;
        };
        if !game.creative && game.native_take_items(&item, 1) == 0 {
            // No item left after all: put the block back.
            if let Some(air) = state_of(game, "minecraft:air") {
                game.native_set_block(&dimension, cell, air, flags::UPDATE_ALL);
            }
            break;
        }
        game.damage_held(1);
    }
}
