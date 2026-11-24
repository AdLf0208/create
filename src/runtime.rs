//! Pure standalone-source model for Create's interop surface (0.1.0).
//!
//! No game access, no network solver: one source block publishes its own saved speed and its
//! declared capacity, carries no stress, and is `Generated` only while it actually spins. The
//! clamping and the exact values come from the cached artefact (see `notes.md`).

/// The published network status of a standalone kinetic source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStatus {
    /// The source is spinning and producing rotation.
    Generated,
    /// The source is stopped; no rotation is produced.
    Idle,
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
        assert_eq!(source_state(MAX_SPEED + 50, MOTOR_CAPACITY).speed, MAX_SPEED);
    }

    #[test]
    fn a_standalone_source_has_no_stress() {
        assert_eq!(source_state(DEFAULT_SPEED, MOTOR_CAPACITY).stress, 0);
        assert_eq!(source_state(0, MOTOR_CAPACITY).stress, 0);
    }
}
