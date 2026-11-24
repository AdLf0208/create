//! Pure kinetic model for Create's interop surface (0.1.0 source, 0.1.1 solver).
//!
//! No game access: the four published values come either from one standalone source block (the
//! 0.1.0 path) or from a bounded solve over a supplied block-id/axis graph (0.1.1). The clamping,
//! the impact values and the exact published numbers come from the cached artefact (see `notes.md`).

/// The published network status of a kinetic component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStatus {
    /// Rotation is being produced and the network is within its stress capacity.
    Generated,
    /// No generator is connected, so no rotation is produced.
    Idle,
    /// A generator is connected, but the connected consumers exceed its capacity.
    Overstressed,
}

/// The four values an add-on reads from a neighbouring Create block entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KineticState {
    /// Rotation speed in RPM, clamped to `0..=MAX_SPEED`.
    pub speed: i32,
    /// Stress capacity in SU, clamped to `0..=MOTOR_CAPACITY`.
    pub capacity: i64,
    /// Stress on a standalone source: always zero (no consumers in this slice).
    pub stress: i64,
    /// `Generated` while spinning, `Idle` at zero speed.
    pub status: NetworkStatus,
}

/// The artefact's declared speed ceiling (Create's 256 RPM maximum).
pub const MAX_SPEED: i32 = 256;
/// The artefact's canonical source speed, from `create.ponder.shared.rpm16_source`.
pub const DEFAULT_SPEED: i32 = 16;
/// The creative motor's declared stress capacity in SU.
pub const MOTOR_CAPACITY: i64 = 16384;

/// The state a standalone source publishes from its configured speed and declared capacity.
///
/// Speed is clamped to `0..=MAX_SPEED` and capacity to `0..=MOTOR_CAPACITY`; a source with no
/// consumers carries no stress and is idle at zero speed. No solver and no network lookup.
#[must_use]
pub fn source_state(configured_speed: i32, capacity: i64) -> KineticState {
    let speed = configured_speed.clamp(0, MAX_SPEED);
    KineticState {
        status: if speed > 0 {
            NetworkStatus::Generated
        } else {
            NetworkStatus::Idle
        },
        speed,
        capacity: capacity.clamp(0, MOTOR_CAPACITY),
        stress: 0,
    }
}

/// A block position in the solver: x, y, z.
pub type Pos = [i32; 3];

/// The six unit steps the solver walks between neighbouring blocks.
const NEIGHBOURS: [Pos; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

/// Bound on one solved component, from the artefact's `MAX_NODES` (architecture §3.1).
pub const MAX_NODES: usize = 4096;

/// The axis a kinetic block rotates around.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// The x axis (east/west).
    X,
    /// The y axis (up/down).
    Y,
    /// The z axis (north/south).
    Z,
}

impl Axis {
    /// Read an `axis` block property (`x`, `y` or `z`).
    #[must_use]
    pub fn from_property(value: &str) -> Option<Self> {
        match value {
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "z" => Some(Self::Z),
            _ => None,
        }
    }

    /// Read a `facing` block property as the axis it points along.
    #[must_use]
    pub fn from_facing(value: &str) -> Option<Self> {
        match value {
            "east" | "west" => Some(Self::X),
            "up" | "down" => Some(Self::Y),
            "north" | "south" => Some(Self::Z),
            _ => None,
        }
    }

    /// The two faces of a shaft on this axis that carry rotation.
    #[must_use]
    pub fn ports(self) -> [Pos; 2] {
        match self {
            Self::X => [[1, 0, 0], [-1, 0, 0]],
            Self::Y => [[0, 1, 0], [0, -1, 0]],
            Self::Z => [[0, 0, 1], [0, 0, -1]],
        }
    }
}

/// The kinetic role of a recognised Create block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KineticKind {
    /// A rotation generator with a configured speed and a stress capacity.
    Motor { speed: i32, capacity: i64 },
    /// A pure transmission block.
    Shaft,
    /// A cogwheel; large cogs accept the 2:1 meshing rule in later slices.
    Cog { large: bool },
    /// A stress consumer (the mechanical press in this slice).
    Press,
}

impl KineticKind {
    /// Stress impact in SU at the artefact's canonical 16 RPM base speed.
    #[must_use]
    pub fn impact(self) -> i64 {
        match self {
            Self::Press => 8,
            Self::Motor { .. } | Self::Shaft | Self::Cog { .. } => 0,
        }
    }
}

/// One block in a kinetic network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KineticNode {
    /// The block's position.
    pub pos: Pos,
    /// The axis it rotates around.
    pub axis: Axis,
    /// What the block does in the network.
    pub kind: KineticKind,
}

/// Recognise the Create block ids this slice can solve into a node.
///
/// `axis_prop` comes from the `axis` block property of shafts and cogs, `facing_prop` from the
/// `facing` property of motors and presses. An unknown id yields `None`: such a block is invisible
/// to the network, which is the artefact's own `owns()` rule (architecture §3.2).
#[must_use]
pub fn node_from_state(
    pos: Pos,
    id: &str,
    axis_prop: &str,
    facing_prop: &str,
    motor_speed: i32,
) -> Option<KineticNode> {
    let (axis, kind) = match id {
        "create:shaft" | "create:encased_shaft" => {
            (Axis::from_property(axis_prop)?, KineticKind::Shaft)
        }
        "create:cogwheel" => (
            Axis::from_property(axis_prop)?,
            KineticKind::Cog { large: false },
        ),
        "create:large_cogwheel" => (
            Axis::from_property(axis_prop)?,
            KineticKind::Cog { large: true },
        ),
        "create:creative_motor" => (
            Axis::from_facing(facing_prop)?,
            KineticKind::Motor {
                speed: motor_speed.clamp(0, MAX_SPEED),
                capacity: MOTOR_CAPACITY,
            },
        ),
        "create:mechanical_press" => (Axis::from_facing(facing_prop)?, KineticKind::Press),
        _ => return None,
    };
    Some(KineticNode { pos, axis, kind })
}

/// Whether a node carries rotation across the face in `delta`.
fn carries(node: KineticNode, delta: Pos) -> bool {
    match node.kind {
        // A press accepts input on any face it is touched from. This is the slice-2
        // simplification of Create's oriented consumer; it is recorded in notes.md.
        KineticKind::Press => true,
        _ => node.axis.ports().contains(&delta),
    }
}

/// Solve the connected component containing `start` into the four published values.
///
/// The graph is supplied by the caller (the game reads block ids, states and the generator's
/// saved speed). This function never touches game state. Speed is the fastest connected
/// generator, capacity is its declared capacity, and stress is the sum of connected consumer
/// impacts scaled by `|speed| / 16` — Create publishes a press as 8 SU at its canonical 16 RPM.
#[must_use]
pub fn solve(start: Pos, lookup: impl Fn(Pos) -> Option<KineticNode>) -> KineticState {
    solve_network(start, lookup).0
}

/// Solve a component and also return every position in it, so the caller can publish the
/// result into each block's own saved data. The order is breadth-first from `start`.
#[must_use]
pub fn solve_network(
    start: Pos,
    lookup: impl Fn(Pos) -> Option<KineticNode>,
) -> (KineticState, Vec<Pos>) {
    let idle = KineticState {
        speed: 0,
        capacity: 0,
        stress: 0,
        status: NetworkStatus::Idle,
    };
    if lookup(start).is_none() {
        return (idle, Vec::new());
    }
    let mut visited: Vec<Pos> = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    visited.push(start);
    queue.push_back(start);
    let mut speed = 0;
    let mut capacity = 0_i64;
    let mut impact = 0_i64;

    while let Some(pos) = queue.pop_front() {
        let Some(node) = lookup(pos) else { continue };
        match node.kind {
            KineticKind::Motor {
                speed: rpm,
                capacity: cap,
            } => {
                speed = speed.max(rpm);
                capacity = cap;
            }
            kind => impact += kind.impact(),
        }
        for delta in NEIGHBOURS {
            if !carries(node, delta) {
                continue;
            }
            let next = [pos[0] + delta[0], pos[1] + delta[1], pos[2] + delta[2]];
            if visited.contains(&next) {
                continue;
            }
            let Some(neighbour) = lookup(next) else {
                continue;
            };
            if !carries(neighbour, [-delta[0], -delta[1], -delta[2]]) {
                continue;
            }
            if visited.len() >= MAX_NODES {
                break;
            }
            visited.push(next);
            queue.push_back(next);
        }
    }

    let stress = impact * i64::from(speed) / i64::from(DEFAULT_SPEED);
    let status = if speed == 0 {
        NetworkStatus::Idle
    } else if stress > capacity {
        NetworkStatus::Overstressed
    } else {
        NetworkStatus::Generated
    };
    (
        KineticState {
            speed,
            capacity,
            stress,
            status,
        },
        visited,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_state_matches_the_motor_values() {
        let state = source_state(DEFAULT_SPEED, MOTOR_CAPACITY);
        assert_eq!(
            state,
            KineticState {
                speed: 16,
                capacity: 16384,
                stress: 0,
                status: NetworkStatus::Generated,
            }
        );
    }

    #[test]
    fn zero_speed_is_idle() {
        let state = source_state(0, MOTOR_CAPACITY);
        assert_eq!(state.speed, 0);
        assert_eq!(state.status, NetworkStatus::Idle);
    }

    #[test]
    fn capacity_is_clamped() {
        assert_eq!(source_state(DEFAULT_SPEED, 99_999).capacity, MOTOR_CAPACITY);
        assert_eq!(source_state(DEFAULT_SPEED, -5).capacity, 0);
        assert_eq!(
            source_state(MAX_SPEED + 50, MOTOR_CAPACITY).speed,
            MAX_SPEED
        );
    }

    #[test]
    fn a_standalone_source_has_no_stress() {
        assert_eq!(source_state(DEFAULT_SPEED, MOTOR_CAPACITY).stress, 0);
        assert_eq!(source_state(0, MOTOR_CAPACITY).stress, 0);
    }
}

/// Slice 2 (0.1.1): the kinetics engine. These are written before the solver exists.
#[cfg(test)]
mod solver_tests {
    use super::*;

    fn lookup(nodes: &[KineticNode]) -> impl Fn(Pos) -> Option<KineticNode> + '_ {
        move |p| nodes.iter().copied().find(|node| node.pos == p)
    }

    fn motor(pos: Pos, speed: i32, capacity: i64) -> KineticNode {
        KineticNode {
            pos,
            axis: Axis::X,
            kind: KineticKind::Motor { speed, capacity },
        }
    }

    fn shaft(pos: Pos) -> KineticNode {
        KineticNode {
            pos,
            axis: Axis::X,
            kind: KineticKind::Shaft,
        }
    }

    fn press(pos: Pos) -> KineticNode {
        KineticNode {
            pos,
            axis: Axis::X,
            kind: KineticKind::Press,
        }
    }

    #[test]
    fn solver_propagates_speed_from_motor_through_shaft_to_press() {
        let nodes = [
            motor([0, 0, 0], 16, 16384),
            shaft([1, 0, 0]),
            press([2, 0, 0]),
        ];
        assert_eq!(
            solve([1, 0, 0], lookup(&nodes)),
            KineticState {
                speed: 16,
                capacity: 16384,
                stress: 8,
                status: NetworkStatus::Generated,
            }
        );
    }

    #[test]
    fn unconnected_shaft_is_idle() {
        let nodes = [shaft([5, 0, 0])];
        assert_eq!(
            solve([5, 0, 0], lookup(&nodes)),
            KineticState {
                speed: 0,
                capacity: 0,
                stress: 0,
                status: NetworkStatus::Idle
            }
        );
    }

    #[test]
    fn stress_sums_every_connected_consumer() {
        let nodes = [
            motor([0, 0, 0], 16, 16384),
            shaft([1, 0, 0]),
            press([2, 0, 0]),
            press([-1, 0, 0]),
        ];
        assert_eq!(solve([1, 0, 0], lookup(&nodes)).stress, 16);
    }

    #[test]
    fn stress_scales_with_speed() {
        let nodes = [
            motor([0, 0, 0], 32, 16384),
            shaft([1, 0, 0]),
            press([2, 0, 0]),
        ];
        assert_eq!(solve([1, 0, 0], lookup(&nodes)).stress, 16);
    }

    #[test]
    fn overstressed_network_is_not_running() {
        let nodes = [motor([0, 0, 0], 16, 4), shaft([1, 0, 0]), press([2, 0, 0])];
        let state = solve([1, 0, 0], lookup(&nodes));
        assert_eq!(state.status, NetworkStatus::Overstressed);
        assert_eq!(
            state.speed, 16,
            "the theoretical speed is kept when overstressed"
        );
    }

    #[test]
    fn perpendicular_axes_do_not_connect() {
        let nodes = [
            motor([0, 0, 0], 16, 16384),
            KineticNode {
                pos: [1, 0, 0],
                axis: Axis::Y,
                kind: KineticKind::Shaft,
            },
            press([1, 1, 0]),
        ];
        assert_eq!(solve([1, 0, 0], lookup(&nodes)).status, NetworkStatus::Idle);
    }

    #[test]
    fn node_recognition_covers_the_five_kinetic_ids_and_refuses_unknowns() {
        assert_eq!(
            node_from_state([3, 4, 5], "create:shaft", "x", "", 0)
                .unwrap()
                .kind,
            KineticKind::Shaft
        );
        assert_eq!(
            node_from_state([3, 4, 5], "create:encased_shaft", "y", "", 0)
                .unwrap()
                .kind,
            KineticKind::Shaft
        );
        assert_eq!(
            node_from_state([3, 4, 5], "create:cogwheel", "z", "", 0)
                .unwrap()
                .kind,
            KineticKind::Cog { large: false }
        );
        assert_eq!(
            node_from_state([3, 4, 5], "create:large_cogwheel", "x", "", 0)
                .unwrap()
                .kind,
            KineticKind::Cog { large: true }
        );
        assert_eq!(
            node_from_state([3, 4, 5], "create:creative_motor", "", "east", 16)
                .unwrap()
                .kind,
            KineticKind::Motor {
                speed: 16,
                capacity: 16384
            }
        );
        assert_eq!(
            node_from_state([3, 4, 5], "create:mechanical_press", "", "west", 0)
                .unwrap()
                .kind,
            KineticKind::Press
        );
        assert!(node_from_state([3, 4, 5], "minecraft:stone", "x", "", 0).is_none());
    }

    #[test]
    fn axis_and_facing_properties_map_to_the_right_axis() {
        assert_eq!(
            node_from_state([0, 0, 0], "create:shaft", "z", "", 0)
                .unwrap()
                .axis,
            Axis::Z
        );
        assert_eq!(
            node_from_state([0, 0, 0], "create:creative_motor", "", "east", 16)
                .unwrap()
                .axis,
            Axis::X
        );
        assert_eq!(
            node_from_state([0, 0, 0], "create:mechanical_press", "", "north", 0)
                .unwrap()
                .axis,
            Axis::Z
        );
    }
}
