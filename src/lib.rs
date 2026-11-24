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

/// The one Part attached to the Creative Motor: its block entity and the tick/publish path.
struct Motor;

#[mod_api::horizon_part]
impl BlockPart for Motor {}

#[mod_api::horizon_part]
impl BlockEntityPart for Motor {
    fn block_entity(&self, _level: LevelSide, event: &mut BlockEntityEvent<'_>) -> Option<Compound> {
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

    fn block_entity_update_tag(&self, event: &mut BlockEntityUpdateTagEvent<'_>) -> Option<Compound> {
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
    }

    /// `/create source <pos>` — the parsed readout an add-on-style consumer uses.
    fn register_commands(&self, host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        let _ = host;
        support::on_register_commands(event);
    }

    fn run_mod_command(&self, host: &mut dyn ServerHost, event: &mut RunModCommandEvent<'_>) -> Flow {
        support::on_run_mod_command(host, event)
    }
}
