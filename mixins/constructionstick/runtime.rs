// Construction Sticks for Horizon — the placement planner (pure).
//
// Ported from Construction Sticks 1.5.0 (Mrbysco/ConstructionSticks, MIT) for
// Minecraft 1.21.1 NeoForge. The upstream source was read for behaviour only;
// this is a clean-room Rust rewrite.
//
// Slice 1 (construction): a stick clicked on a block fills the plane of the
// clicked face with copies of the clicked block, growing outward from the
// clicked position, bounded by the tier's block limit and the player's supply.
// Each placed block consumes one item of the clicked block's kind and wears the
// stick by one. Angel/destruction/replacement modes, upgrades and the options
// screen are later slices (see notes.md).
//
// This file is pure logic so it compiles and unit-tests as the crate's
// standalone library; the game-side glue lives in the bundle's support module.

pub const WOODEN_STICK: &str = "constructionstick:wooden_stick";
pub const COPPER_STICK: &str = "constructionstick:copper_stick";
pub const IRON_STICK: &str = "constructionstick:iron_stick";
pub const DIAMOND_STICK: &str = "constructionstick:diamond_stick";
pub const NETHERITE_STICK: &str = "constructionstick:netherite_stick";

/// One stick tier: how many blocks one use may place, and its durability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stick {
    pub limit: u32,
    pub durability: u32,
}

/// The tier of a held stack by item id.
pub fn stick_for(item: &str) -> Option<Stick> {
    let (limit, durability) = match item {
        WOODEN_STICK => (3, 59),
        COPPER_STICK => (9, 131),
        IRON_STICK => (27, 250),
        DIAMOND_STICK => (128, 1561),
        NETHERITE_STICK => (1024, 2031),
        _ => return None,
    };
    Some(Stick { limit, durability })
}

/// What the planner reads from the world.
pub trait World {
    /// The block at `pos`, by name; None when it is not observable (unloaded,
    /// out of bounds).
    fn block(&mut self, pos: [i32; 3]) -> Option<String>;
    /// Whether a block may be placed at `pos`: air (or a replaceable block)
    /// with no living entity in the way. The glue decides.
    fn can_place(&mut self, pos: [i32; 3]) -> bool;
}

/// The eight in-plane directions for a face: both ways on each of the two
/// axes, plus the four diagonals (upstream's `nolock` gate).
pub fn plane_offsets(normal: [i32; 3]) -> [[i32; 3]; 8] {
    let axis = if normal[1] != 0 {
        1
    } else if normal[0] != 0 {
        0
    } else {
        2
    };
    match axis {
        1 => [
            [1, 0, 0],
            [-1, 0, 0],
            [0, 0, 1],
            [0, 0, -1],
            [1, 0, 1],
            [1, 0, -1],
            [-1, 0, 1],
            [-1, 0, -1],
        ],
        0 => [
            [0, 0, 1],
            [0, 0, -1],
            [0, 1, 0],
            [0, -1, 0],
            [0, 1, 1],
            [0, 1, -1],
            [0, -1, 1],
            [0, -1, -1],
        ],
        _ => [
            [1, 0, 0],
            [-1, 0, 0],
            [0, 1, 0],
            [0, -1, 0],
            [1, 1, 0],
            [1, -1, 0],
            [-1, 1, 0],
            [-1, -1, 0],
        ],
    }
}

/// Whether a resolved vanilla placement (`Option<[i32; 3]>` from
/// `BlockItem.getStateForPlacement` onward) lands in the planned `cell`.
///
/// Vanilla puts the block where it resolved: the cell itself when the cell is
/// air or replaceable, else beside it. Only the first is a plane cell, so a
/// placement that resolves elsewhere is refused rather than placed out of the
/// plane.
pub fn accepts(cell: [i32; 3], resolved: Option<[i32; 3]>) -> bool {
    resolved == Some(cell)
}

/// The cells one use of a stick places, in placement order.
///
/// `clicked` is the block that was right-clicked; the plane starts one step
/// along `normal` (the clicked face) and only grows where the block behind a
/// cell (cell − normal) is still the same block as `support`, as upstream does.
/// At most `limit` cells are returned.
pub fn plan(
    world: &mut impl World,
    clicked: [i32; 3],
    normal: [i32; 3],
    support: &str,
    limit: u32,
) -> Vec<[i32; 3]> {
    if normal == [0, 0, 0] || limit == 0 {
        return Vec::new();
    }
    let start = [
        clicked[0] + normal[0],
        clicked[1] + normal[1],
        clicked[2] + normal[2],
    ];
    let offsets = plane_offsets(normal);
    let mut queue = std::collections::VecDeque::from([start]);
    let mut visited = std::collections::HashSet::new();
    let mut out = Vec::new();
    while let Some(pos) = queue.pop_front() {
        if out.len() >= limit as usize {
            break;
        }
        if !visited.insert(pos) {
            continue;
        }
        let behind = [pos[0] - normal[0], pos[1] - normal[1], pos[2] - normal[2]];
        if world.block(behind).as_deref() != Some(support) {
            continue;
        }
        if !world.can_place(pos) {
            continue;
        }
        out.push(pos);
        for offset in offsets {
            queue.push_back([pos[0] + offset[0], pos[1] + offset[1], pos[2] + offset[2]]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    struct TestWorld {
        blocks: HashMap<[i32; 3], String>,
        blocked: HashSet<[i32; 3]>,
    }

    impl TestWorld {
        fn new() -> Self {
            TestWorld {
                blocks: HashMap::new(),
                blocked: HashSet::new(),
            }
        }
        fn stone(&mut self, pos: [i32; 3]) {
            self.blocks.insert(pos, "minecraft:stone".into());
        }
        fn occupied(&mut self, pos: [i32; 3]) {
            self.blocked.insert(pos);
        }
    }

    impl World for TestWorld {
        fn block(&mut self, pos: [i32; 3]) -> Option<String> {
            self.blocks.get(&pos).cloned()
        }
        fn can_place(&mut self, pos: [i32; 3]) -> bool {
            !self.blocked.contains(&pos) && !self.blocks.contains_key(&pos)
        }
    }

    #[test]
    fn tiers_match_the_five_sticks() {
        assert_eq!(
            stick_for(WOODEN_STICK),
            Some(Stick {
                limit: 3,
                durability: 59
            })
        );
        assert_eq!(
            stick_for(COPPER_STICK),
            Some(Stick {
                limit: 9,
                durability: 131
            })
        );
        assert_eq!(
            stick_for(IRON_STICK),
            Some(Stick {
                limit: 27,
                durability: 250
            })
        );
        assert_eq!(
            stick_for(DIAMOND_STICK),
            Some(Stick {
                limit: 128,
                durability: 1561
            })
        );
        assert_eq!(
            stick_for(NETHERITE_STICK),
            Some(Stick {
                limit: 1024,
                durability: 2031
            })
        );
        assert_eq!(stick_for("minecraft:stick"), None);
        assert_eq!(stick_for("constructionstick:template_angel"), None);
    }

    #[test]
    fn a_wall_face_fills_its_plane_outward() {
        let mut world = TestWorld::new();
        for x in -2..=2 {
            for y in 63..=65 {
                world.stone([x, y, 0]);
            }
        }
        let placed = plan(&mut world, [0, 64, 0], [0, 0, 1], "minecraft:stone", 27);
        assert_eq!(placed.len(), 15, "5 wide x 3 high stone wall");
        assert!(placed.contains(&[0, 64, 1]));
        assert!(placed.contains(&[-2, 63, 1]));
        assert!(placed.contains(&[2, 65, 1]));
        assert!(placed.iter().all(|p| p[2] == 1), "one plane only");
    }

    #[test]
    fn the_tier_limit_caps_the_fill() {
        let mut world = TestWorld::new();
        for x in -5..=5 {
            for y in 60..=70 {
                world.stone([x, y, 0]);
            }
        }
        assert_eq!(
            plan(&mut world, [0, 64, 0], [0, 0, 1], "minecraft:stone", 3).len(),
            3
        );
        assert_eq!(
            plan(&mut world, [0, 64, 0], [0, 0, 1], "minecraft:stone", 9).len(),
            9
        );
    }

    #[test]
    fn a_gap_in_the_support_stops_the_growth() {
        let mut world = TestWorld::new();
        for x in -2..=2 {
            world.stone([x, 64, 0]);
        }
        // A hole behind (2,64,1) means that cell and everything past it fail.
        world.blocks.remove(&[2, 64, 0]);
        let placed = plan(&mut world, [0, 64, 0], [0, 0, 1], "minecraft:stone", 27);
        assert!(!placed.contains(&[2, 64, 1]), "no support behind the cell");
        assert!(!placed.contains(&[3, 64, 1]), "growth cannot pass the gap");
        assert!(placed.contains(&[-2, 64, 1]));
    }

    #[test]
    fn an_occupied_cell_is_skipped_and_does_not_forward() {
        let mut world = TestWorld::new();
        for x in -2..=2 {
            world.stone([x, 64, 0]);
        }
        world.occupied([1, 64, 1]);
        let placed = plan(&mut world, [0, 64, 0], [0, 0, 1], "minecraft:stone", 27);
        assert!(!placed.contains(&[1, 64, 1]));
        assert!(placed.contains(&[0, 64, 1]));
        assert!(placed.contains(&[-1, 64, 1]));
    }

    #[test]
    fn a_ceiling_fills_the_horizontal_plane_below() {
        let mut world = TestWorld::new();
        for x in -1..=1 {
            for z in -1..=1 {
                world.stone([x, 65, z]);
            }
        }
        let placed = plan(&mut world, [0, 65, 0], [0, -1, 0], "minecraft:stone", 9);
        assert_eq!(placed.len(), 9);
        assert!(placed.iter().all(|p| p[1] == 64));
    }

    #[test]
    fn a_placement_must_land_in_the_planned_cell() {
        // Air (or a replaceable block): vanilla resolves the placement in the
        // cell, so the plane may take it.
        assert!(accepts([0, 64, 1], Some([0, 64, 1])));
        // An occupied cell resolves beside it: placing there would leave the
        // clicked face's plane, so it is refused.
        assert!(!accepts([0, 64, 1], Some([0, 64, 2])));
        // Vanilla refused the placement outright.
        assert!(!accepts([0, 64, 1], None));
    }

    #[test]
    fn a_different_support_block_is_not_copied() {
        let mut world = TestWorld::new();
        world.stone([0, 64, 0]);
        world.blocks.insert([1, 64, 0], "minecraft:dirt".into());
        let placed = plan(&mut world, [0, 64, 0], [0, 0, 1], "minecraft:stone", 27);
        assert_eq!(placed, vec![[0, 64, 1]]);
    }
}
