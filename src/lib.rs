//! Create for Horizon — the one `impl Hooks` at the crate root.
//!
//! The pure model lives in [`runtime`], with no game types, so the published numbers can be
//! unit-tested with `cargo test` alone. This file registers one Creative Motor block entity whose
//! saved compound publishes `Speed`, `Capacity`, `Stress` and `NetworkStatus` — the world-level
//! shape a dependent add-on reads from a neighbouring Create block. No cross-mod Rust API.
pub mod runtime;
mod support;

use std::sync::Arc;

use mod_api::{
    BlockEntityEvent, BlockEntityPart, BlockEntityTickEvent, BlockEntityTicksEvent,
    BlockEntityUpdateTagEvent, BlockPart, Compound, Flow, Hooks, Host, LevelSide,
    RegisterCommandsEvent, Registrar, RegistrationContext, RunModCommandEvent, ServerHost,
};

/// The source block that publishes the interop readout.
pub(crate) const MOTOR: &str = "create:creative_motor";
/// Transmission and consumer ids the 0.1.1 solver recognises.
pub(crate) const SHAFT: &str = "create:shaft";
pub(crate) const COGWHEEL: &str = "create:cogwheel";
pub(crate) const LARGE_COGWHEEL: &str = "create:large_cogwheel";
pub(crate) const ENCASED_SHAFT: &str = "create:encased_shaft";
pub(crate) const PRESS: &str = "create:mechanical_press";
/// Every carried id that gets a kinetic block entity.
const KINETIC_IDS: [&str; 5] = [SHAFT, COGWHEEL, LARGE_COGWHEEL, ENCASED_SHAFT, PRESS];

/// The one Part attached to the Creative Motor: its block entity and the tick/publish path.
struct Motor;

#[mod_api::horizon_part]
impl BlockPart for Motor {}

#[mod_api::horizon_part]
impl BlockEntityPart for Motor {
    fn block_entity(
        &self,
        _level: LevelSide,
        event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        (event.block == MOTOR).then(support::initial_data)
    }

    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        if event.block == MOTOR {
            event.ticks = true;
        }
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let _ = host;
        support::on_block_entity_tick(event);
    }

    fn block_entity_update_tag(
        &self,
        event: &mut BlockEntityUpdateTagEvent<'_>,
    ) -> Option<Compound> {
        Some(support::published_fields(&event.entity.nbt))
    }
}

/// The transmission/consumer blocks: their saved compound carries the solved four fields.
struct Kinetic;

#[mod_api::horizon_part]
impl BlockPart for Kinetic {}

#[mod_api::horizon_part]
impl BlockEntityPart for Kinetic {
    fn block_entity(
        &self,
        _level: LevelSide,
        event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        KINETIC_IDS.contains(&event.block).then(support::idle_data)
    }

    fn block_entity_update_tag(
        &self,
        event: &mut BlockEntityUpdateTagEvent<'_>,
    ) -> Option<Compound> {
        Some(support::published_fields(&event.entity.nbt))
    }
}

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Attach the motor's block-entity Part to the block declared in `data/create/horizon/block/`.
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let kind = r.declare_block_entity_type("motor", vec![MOTOR.to_owned()]);
        if let Err(error) = r.attach_block_entity_part(&kind, "motor", Arc::new(Motor)) {
            support::warn_refused(error);
        }
        let kinetic = r.declare_block_entity_type(
            "kinetic",
            KINETIC_IDS.iter().map(|id| (*id).to_owned()).collect(),
        );
        if let Err(error) = r.attach_block_entity_part(&kinetic, "kinetic", Arc::new(Kinetic)) {
            support::warn_refused(error);
        }
    }

    /// `/create source <pos>` — the parsed readout an add-on-style consumer uses.
    fn register_commands(&self, host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        let _ = host;
        support::on_register_commands(event);
    }

    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        support::on_run_mod_command(host, event)
    }
}
