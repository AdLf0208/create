//! Game-side glue for Create's interop surface (0.1.0).
//!
//! The motor's saved compound holds its configured `Speed` and the published `Capacity`, `Stress`
//! and `NetworkStatus`. Every server tick rebuilds the pure [`source_state`] from the saved speed
//! and writes the four fields back — the same NBT shape an add-on reads from a neighbour with
//! `native_with_mod_block_entity`. A standalone source has no consumers, so after the first tick the
//! payload is stable and the block answers `Unchanged` (an idle motor does not dirty its chunk).
//!
//! No network solver, no propagation, no cross-mod Rust symbol: those are 0.1.1.

use mod_api::{
    BlockEntityTickEvent, BlockEntityTickReply, Command, CommandArgumentValue, CommandInvocation,
    Compound, DimensionKey, Flow, Host, LevelRef, RegisterCommandsEvent,
    RunModCommandEvent, ServerHost, Tag, Text,
};

use crate::MOTOR;
use crate::runtime::{DEFAULT_SPEED, MOTOR_CAPACITY, source_state};

/// published NBT keys — the exact shape add-ons consume (architecture doc §2.4).
const SPEED: &str = "Speed";
const CAPACITY: &str = "Capacity";
const STRESS: &str = "Stress";
const STATUS: &str = "NetworkStatus";

/// The status strings a Create network publishes.
const RUNNING: &str = "Running";
const IDLE: &str = "Idle";

/// A newly placed motor starts at the artefact's canonical source speed; the first tick
/// publishes `Generated`/`Running` and the rest of the fields.
pub(crate) fn initial_data() -> Compound {
    let mut data = Compound::new();
    put_f32(&mut data, SPEED, DEFAULT_SPEED as f32);
    write_fields(&mut data, DEFAULT_SPEED);
    data
}

/// One tick: derive the four fields from the saved speed and write them back in place.
pub(crate) fn on_block_entity_tick(event: &mut BlockEntityTickEvent<'_>) {
    let saved = event.entity.nbt.clone();
    let speed = read_f32(&saved, SPEED).unwrap_or(DEFAULT_SPEED as f32);
    let speed_i32 = speed.round() as i32;
    let mut data = saved;
    write_fields(&mut data, speed_i32);
    if data != event.entity.nbt {
        event.entity.nbt = data;
    } else {
        *event.reply = BlockEntityTickReply::Unchanged;
    }
}

/// The four published fields, as a standalone client snapshot and for the command readout.
pub(crate) fn published_fields(nbt: &Compound) -> Compound {
    let mut tag = Compound::new();
    for key in [SPEED, CAPACITY, STRESS, STATUS] {
        if let Some(value) = field(nbt, key) {
            tag.insert(key.into(), value.clone());
        }
    }
    tag
}

/// Read a compound field by its UTF-8 key (`Compound` is keyed by UTF-16 `JavaString`).
fn field<'a>(data: &'a Compound, key: &str) -> Option<&'a Tag> {
    data.iter()
        .find(|(name, _)| name.to_string_lossy() == key)
        .map(|(_, value)| value)
}

/// Compute and store the four values from a configured speed (the pure model's contract).
fn write_fields(data: &mut Compound, configured_speed: i32) {
    let state = source_state(configured_speed, MOTOR_CAPACITY);
    put_f32(data, SPEED, state.speed as f32);
    put_f32(data, CAPACITY, state.capacity as f32);
    put_f32(data, STRESS, state.stress as f32);
    let name = match state.status {
        crate::runtime::NetworkStatus::Generated => RUNNING,
        crate::runtime::NetworkStatus::Idle => IDLE,
    };
    data.insert(STATUS.into(), Tag::String(name.into()));
}

/// NBT stores an f32 as its raw IEEE-754 bits (`Tag::Float(u32)`).
fn put_f32(data: &mut Compound, key: &str, value: f32) {
    data.insert(key.into(), Tag::Float(value.to_bits()));
}

fn read_f32(data: &Compound, key: &str) -> Option<f32> {
    match field(data, key) {
        Some(Tag::Float(bits)) => Some(f32::from_bits(*bits)),
        _ => None,
    }
}

fn read_i64(data: &Compound, key: &str) -> Option<i64> {
    match field(data, key) {
        Some(Tag::Float(bits)) => Some(f32::from_bits(*bits) as i64),
        Some(Tag::Long(value)) => Some(*value),
        Some(Tag::Int(value)) => Some(i64::from(*value)),
        _ => None,
    }
}

fn read_status(data: &Compound) -> String {
    match field(data, STATUS) {
        Some(Tag::String(text)) => text.to_string_lossy(),
        _ => IDLE.to_owned(),
    }
}

pub(crate) fn warn_refused(error: impl std::fmt::Display) {
    eprintln!("Create: the motor block-entity part was refused: {error} -- the readout will not run");
}

/// `/create source <pos>`.
pub(crate) fn on_register_commands(event: &mut RegisterCommandsEvent<'_>) {
    event.commands.push(Command::new("create source <pos:block_pos>"));
}

/// Reply lines are parsed by `tools/e2e.sh`; the prefix names this mod's readout.
fn reply(host: &mut dyn ServerHost, line: &str) {
    let _ = host.commands().feedback(Text::literal(line));
}

pub(crate) fn on_run_mod_command(
    host: &mut dyn ServerHost,
    event: &mut RunModCommandEvent<'_>,
) -> Flow {
    let invocation = event.command;
    let mut literals = invocation.literals.iter().map(String::as_str);
    if literals.next() != Some("create") || literals.next() != Some("source") {
        return Flow::Continue;
    }
    let Some(pos) = block_pos(invocation, "pos") else {
        reply(host, "CREATE_SOURCE refused: usage /create source <x> <y> <z>");
        return Flow::Cancel;
    };
    let Some(level) = level(host, &invocation.dimension) else {
        reply(host, "CREATE_SOURCE refused: dimension not loaded");
        return Flow::Cancel;
    };
    let Some(block) = level.block(&*host, pos) else {
        reply(host, "CREATE_SOURCE refused: position not loaded");
        return Flow::Cancel;
    };
    if block.id(&*host) != MOTOR {
        reply(
            host,
            &format!("CREATE_SOURCE refused: no Create block entity at {pos:?}"),
        );
        return Flow::Cancel;
    }
    let Some(entity) = host.block_entity(level, pos) else {
        reply(host, "CREATE_SOURCE refused: no block entity at that position");
        return Flow::Cancel;
    };
    let data = entity.data(&*host).unwrap_or_default();
    let speed = read_i64(&data, SPEED).unwrap_or(0);
    let capacity = read_i64(&data, CAPACITY).unwrap_or(0);
    let stress = read_i64(&data, STRESS).unwrap_or(0);
    let status = read_status(&data);
    reply(
        host,
        &format!(
            "CREATE_SOURCE x={} y={} z={} speed={speed} capacity={capacity} stress={stress} status={status}",
            pos[0], pos[1], pos[2]
        ),
    );
    Flow::Cancel
}

fn block_pos(invocation: &CommandInvocation, name: &str) -> Option<[i32; 3]> {
    match invocation.argument(name) {
        Some(CommandArgumentValue::BlockPos(pos)) => Some(*pos),
        _ => None,
    }
}

fn level(host: &dyn Host, dimension: &DimensionKey) -> Option<LevelRef> {
    dimension.id(host).and_then(|id| host.level(&id))
}
