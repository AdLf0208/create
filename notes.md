# Create for Horizon — port notes (slice 2, `0.1.1` — kinetics engine)

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-review

- **Class / tier (ledger):** `content` / `deep`. The tier's honest shape for slice 1 is the **machines**
  part of Create: one block entity whose saved compound publishes the kinetic readout add-ons consume.
  The rest of the deep tier (entities/AI, contraptions, trains, Ponder, schematics) is out of scope.
- **Approach (from `horizon/docs/modpacks.md`):** carry the registered ids and their regenerated art;
  the agent part is the pure `runtime.rs` model plus the block-entity/readout glue. No cross-mod Rust
  API — interop is at the world/data level (block ids/states and the neighbour's published NBT).
- **Pin:** Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` + mixins
  `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`; Minecraft 26.3 (from `26.3.zip`).
- **Licence:** MIT code / **All Rights Reserved assets** under `src/main/resources/assets/`
  (`assets-arr`, code rank 0). Verified from `Creators-of-Create/Create/LICENSE.md` (primary source).
  Every art byte is regenerated; **zero jar-derived asset bytes** proved by `tools/check_provenance.py
  --check`. MIT text, a NOTICE crediting The Create Team and linking
  `https://github.com/Creators-of-Create/Create`, and a monogram icon ship with the package.
- **Artefact authority:** the cached `create-1.21.1-6.0.10.jar` is the behaviour/data authority. It is
  never a build target; all builds and e2e run against the pinned 26.3 SDK.

## Slice 2 (`0.1.1`) — the kinetics engine

This slice replaces 0.1.0's documented "no solver" limitation. New carried blocks:
`create:shaft`, `create:cogwheel`, `create:large_cogwheel`, `create:encased_shaft` (each with an
`axis` property) and `create:mechanical_press` (with `facing`). All five get a block entity whose
saved compound carries the same four published fields. Creative Motor stays the generator.

### Solver rules as read (clean-room, from `ports/_shared/external/create-horizon-architecture.md`)

- §3.1: the upper bounds are `MAX_NODES = 4096` and `MAX_RPM = 256`; `Status` is
  `Idle`/`Running`/`Overstressed` (plus conflict/cycle/overflow statuses this slice does not reach).
  Stress is the sum of consumer impacts over the network; a press's impact is **8 SU**.
- §3.2: block id → node recognition is the *only* place a block enters the graph; any id outside
  the list is invisible (`owns()`). Axis comes from the `axis` property, falling back to `facing`.
- §3.3: the network is re-derived from world block ids/states, and **every vertex carries the same
  solved rotation in its own NBT** after a solve. An add-on reads a neighbour's published fields;
  it never calls a Rust symbol (§4.4 forbids the incidental path).
- §3.1 impact is published **at the artefact's canonical 16 RPM** (`create.ponder.shared.rpm16_source`
  in the jar's `en_us.json`, already the 0.1.0 source value). The port therefore computes
  `stress = Σ impact × |speed| / 16` in integer arithmetic: a press at 16 RPM charges 8 SU, at 32 RPM
  charges 16 SU. Both cases are pinned by unit tests and the in-game readout.

Stated simplifications (clean-room re-derivation, not Java translation):
- coaxial shafts/cogs connect at distance 1 on their shared axis; the perpendicular large-cog
  2:1 meshing rule and belts/chains/clutches/gearshifts (§3.1 rules 2–5) are out of scope;
- a press accepts rotation from any touching face instead of only its oriented back face;
- the motor has ports on both ends of its facing axis (no directional output check yet);
- conflicting generators take the maximum speed instead of a `ConflictingSources` failure.

**SDK deviation (recorded, not hidden):** the tick hook `BlockEntityTickEvent` exposes only `pos`
and the entity view (`ports/_shared/sdk/0a875da/.../generated/mod_api_generated.rs:10064`) — no
`LevelRef` — so a tick cannot read neighbouring blocks. The solve therefore runs on the
`/create network <pos>` query path, which has the caller's dimension, and publishes into **every
node's** saved data with `BlockEntityRef::merge_data` (`mod_api_generated_host.rs:87`). This is the
same world/data contract an add-on consumes; per-tick re-solving is a later gap, not a fake.

## Inventory of the cached jar (measured with `unzip -Z1`, before choosing any family)

| Family | Count (jar) |
|---|---:|
| total entries | 12,536 |
| blockstates (`assets/create/blockstates/`) | 644 |
| item models (`assets/create/models/item/`) | 785 |
| loot tables (`data/create/loot_table/`) | 642 |
| recipes (`data/create/recipe/`) | 1,990 |
| tags (`data/{create,c}/tags/`) | 183 |
| lang files | 8 (`en_us.json` has 3,639 keys) |

Kinetic-source blockstates present: `creative_motor`, `hand_crank`, `water_wheel`,
`large_water_wheel`, `rotation_speed_controller`, `shaft`, `cogwheel`/`large_cogwheel` (item models),
`gearbox`, `gearshift`, `speedometer`, `stressometer`. The `creative_motor` blockstate declares only
the six `facing` variants (its RPM is BE data, `ScrollValue`, set in a GUI — not a block property).

Artefact values used by the pure model (with evidence):
- `MAX_SPEED = 256` RPM — Create's declared speed ceiling (`MAX_RPM 256.0`, measured in
  `ports/_shared/external/create-horizon-architecture.md` §3.1; the speed controller's max).
- `DEFAULT_SPEED = 16` RPM — the artefact's own canonical source value
  (`create.ponder.shared.rpm16_source = "Source: 16 RPM"` and `create.ponder.shared.rpm16 = "16 RPM"`
  in `assets/create/lang/en_us.json`). Slice 1 fixes the standalone source at this value; no GUI ships
  yet, so the value is saved in the block entity's NBT.
- `MOTOR_CAPACITY = 16384` SU — the creative motor's declared stress capacity (architecture §3.2).
- `stress = 0` for a standalone source; `NetworkStatus = Generated` at positive speed, `Idle` at zero.

## Carried-id contract (decided before the carry)

Criterion: carry only ids named by dependents that are rank 0/1, not `code-arr`, and not already blocked
on a missing parent surface, plus the recipe ingredients/tags needed to use them.

**Viable dependents considered** (from `dependencies.csv`, `licence-classes.csv` and architecture §5):
`createaddition` (MIT, 0), `create_hypertube` (Apache-2.0, 0), `create_aquatic_ambitions` (MIT, 0),
`create_dragons_plus` (LGPL-3.0, 1), `bellsandwhistles` (GPL-3.0, 1).
**Excluded dependents:** `copycats`, `rechiseledcreate`, `sophisticatedbackpackscreateintegration`,
`sophisticatedstoragecreateintegration` (rank-4 `code-arr`, never ported); `ars_creo` (blocked on the
kinetics engine + parent `ars_nouveau` partial); `create_enchantment_industry` (blocked on
`create_dragons_plus`), plus `colorwheel`/`invasiveopts` (optional, no named Create id).

**Carried set (exact, 0.1.1 adds the five kinetic blocks):**
- blocks: `create:creative_motor` (generator, 0.1.0) plus `create:shaft`, `create:cogwheel`,
  `create:large_cogwheel`, `create:encased_shaft`, `create:mechanical_press` (slice 2) — each block
  entity publishes the four solved fields.
- items: `create:creative_motor` (block item), `create:andesite_alloy`, `create:brass_ingot`,
  `create:brass_nugget`, `create:iron_sheet`, `create:brass_sheet`, `create:copper_sheet` — the
  material ids named by `bellsandwhistles`' recipes (`create:andesite_alloy`, `create:brass_ingot/nugget`,
  `create:*_sheet`) and by Create's own kinetic blocks.
- recipe: `create:andesite_alloy` (crafting_shaped, `minecraft:andesite` + `c:nuggets/iron`) — gives the
  carried alloy item a craft in game. The brass/sheet items have no vanilla craft in the artefact
  (mixer/press only, deferred to the kinetics slice); they ship as registered recipe-ingredient ids.
- loot table: `create:creative_motor` (self-drop).
- tag: `c:nuggets/iron` → `minecraft:iron_nugget` (the alloy recipe's ingredient tag).

**Excluded from the carry:** belts, trains, Ponder UI, schematics, contraptions, the remaining ~640
blockstates and ~1,990 recipes, all ARR textures/lang/models from the jar. `hand_crank`, `water_wheel`
and `large_water_wheel` need an environment model (water current / player use) the standalone slice
does not have; they are deferred to the kinetics slice.

`tools/carry_contract.json` records the counted set; `tools/inventory.py --check` fails on an empty
tree and passes on exactly this set.

## Coverage table (parity rule)

| Family | Status | Proof |
|---|---|---|
| block `create:creative_motor` | tested | `ctl setblock` → `ctl block` read back `create:creative_motor` (green e2e, phase A) |
| item ids (7) | tested | `ctl give create:andesite_alloy 2` → `ctl inventory` count 2 (phase C); unknown id refused in the control |
| recipe `create:andesite_alloy` | tested | survival 2×2 grid `BA/AB`: result slot `create:andesite_alloy`, 2 andesite + 2 iron nuggets consumed (4→2 each), alloy taken to the inventory (phase D) |
| loot table `creative_motor` | tested (data) | 26.3 dialect accepted by `check_261_dialect.py`; self-drop entry and `package matches the tree` gate |
| tag `c:nuggets/iron` | tested | the craft in phase D resolves its ingredient through the declared tag |
| BE readout `Speed`/`Capacity`/`Stress`/`NetworkStatus` | tested | parsed `/create source 300 101 0` → exact `CREATE_SOURCE … speed=16 capacity=16384 stress=0 status=Running`; vanilla block refusal; no-mod control `EXIT:1` |
| regenerated art / monogram icon / MIT+NOTICE | tested | `check_provenance.py --check`: 0 matches; `gen_assets.py --check` reproducible; LICENSE + NOTICE packed |
| node recognition (`shaft`, `encased_shaft`, cogs, motor, press) | tested | unit `node_recognition_covers_the_five_kinetic_ids_and_refuses_unknowns`; in-game setblock/read-back of `create:shaft[axis=x]` and the press (phase E) |
| propagation along connected axes | tested | unit `solver_propagates_speed_from_motor_through_shaft_to_press` and `perpendicular_axes_do_not_connect`; phase E motor→shaft→press at 16 RPM |
| stress accounting `Σ impact × \|speed\| / 16` | tested | unit `stress_sums_every_connected_consumer`, `stress_scales_with_speed`; phase E `CREATE_NETWORK … stress=8`, `SELFTEST=stress9` `EXIT:1` |
| mechanical press *pressing* behaviour | not ported | 0.1.2: the press is a stress node in 0.1.1, but no item transform runs yet |
| unconnected shaft is idle | tested | unit `unconnected_shaft_is_idle`; phase E control shaft at (305,101,0) → `speed=0 … status=Idle` |
| overstress status | tested (unit) | `overstressed_network_is_not_running`: capacity 4, stress 8 → `Overstressed`, theoretical speed kept |
| neighbour BE publish (cross-mod data feed) | tested (in-port) | parsed `CREATE_NETWORK published …` read back from the shaft's saved NBT; the full add-on-side demonstration is the **cross-mod gate** below |
| Ponder / trains / contraptions / schematics | not ported | out of scope (deep tier) |
| belts / chains / clutches / gearshift ratios | not ported | §3.1 rules 2–5; bounded slice, stated above |
| per-tick re-solve in `block_entity_tick` | blocked(no-level-in-tick-event) | the tick event exposes no `LevelRef`; solve runs on the query path and publishes through `merge_data` |

## Upstream defects fixed

None found in slice 1. The creative motor's RPM is GUI-configured and has no blockstate default; the
port fixes the standalone source at the artefact's own canonical 16 RPM and records that choice above.

**Slice 2 (clean-room deviations, not Java defects):** the four simplifications and the tick-event
gap are listed under "Solver rules as read" above. None is a defect in the artefact — each is a
bounded scope choice forced by the SDK surface or by the slice's tier — and each is pinned by a unit
test or an in-game assertion rather than hidden.

**Cross-mod demonstration gate (active):** the add-on half of the interop contract needs a dependent
port's write ownership (a `bellsandwhistles`/`create_aquatic_ambitions` slice reading a neighbouring
Create shaft's published fields). This job owns only `ports/create/**`, so the gate is reported and
the assertion is kept inside this port's parsed `CREATE_NETWORK published …` readout, which proves
the same world/data feed an add-on would consume. No cross-mod Rust symbol is used.

## Verification (re-read from the artefacts)

- **Delivered package:** `create.hmod`, **34,810 B**, sha256
  `bec224ec6802c113b0082bbbce78627f523770005333455ebb523f09c2a41b90`, **68 entries**, no
  `Cargo.lock`. `cargo horizon-mod check`: `native-mod, format native-1/0a875da…/e59da90…, for 26.3`;
  it reports 172 API items used and "made for 0a875da".
- **Red, verbatim** (`ports/_shared/evidence/create/sdk-test-red-solver.log`, the 0.1.1 solver
  tests written before the solver existed): `error[E0425]: cannot find function `solve` in this scope`
  (7 occurrences), `error[E0425]: cannot find function `node_from_state` in this scope` (9),
  `error[E0599]: no variant or associated item named `Overstressed` found for enum
  `runtime::NetworkStatus``, ending `error: could not compile `create` (lib test) due to 47 previous
  errors` — exit 101.
- **Green tests / lints** (`sdk-test.log`, `final-clippy.log`, `final-fmt.log`, regenerated after the
  final package build): **12 passed / 0 failed** (8 new solver tests on top of 0.1.0's 4); clippy
  `--all-targets -- -D warnings` exit 0; `cargo fmt --check` exit 0.
- **Both `fits` lines on the delivered sha** (`sdk-fits.log`, `sdk-fits-other.log`):
  `0a875da  made for` (exit 0) and `0b14f8a  fits` (exit 0). Both were run inside a build-slot
  lease (`--step run`, package metadata only — no compile) because the remote box has no cached
  SDK release for these two builds.
- **Zero jar-derived asset bytes** (`check_provenance.py --check`):
  `OK: zero jar-derived asset bytes — 59 candidate files, 68 packaged files, 0 matches against
  11548 jar entries`. `PROVENANCE.md` is the per-file report; `gen_assets.py --check` re-renders
  all 59 generated files byte-for-byte, including the five new kinetic blocks.
- **SDK gate:** `verify-sdk-package.py create` → **1/1** (package sha, `runtime.rs` byte-identical
  `5d60fbfb8578`, `package matches the tree`, R9 tables/deps/build-script, no lock, 12 tests,
  clippy, check, fits, skills gate).
- **Dialect:** `check_261_dialect.py ports/create` → both recipe and loot table clean.
- **In-game transcript** (green `e2e-launcher.log`, launcher-built game at the pin):
  phase A `CREATE_SOURCE x=300 y=101 z=0 speed=16 capacity=16384 stress=0 status=Running`; phase B
  `CREATE_SOURCE refused: no Create block entity at [304, 101, 0]`; phase D result
  `create:andesite_alloy` with `{"create:andesite_alloy":1, "minecraft:andesite":2,
  "minecraft:iron_nugget":2}`; phase E `ctl block` read-back of all four placed ids, then
  `CREATE_NETWORK x=301 y=101 z=0 speed=16 stress=8 capacity=16384 status=Running`, the neighbour
  read-back `CREATE_NETWORK published x=301 y=101 z=0 speed=16 stress=8 capacity=16384
  status=Running`, the idle control `CREATE_NETWORK x=305 y=101 z=0 speed=0 stress=0 capacity=0
  status=Idle`, and the differential after placing an out-of-component press at (304,101,0)
  still `stress=8`. `PASS … EXIT:0`.
- **Control** (`e2e-launcher-control.log`, unmodded Nightly): `Unknown block type
  'create:creative_motor'`, `Unknown or incomplete command … create source …`, `FAIL … EXIT:1`.
- **Selftest** (`e2e-launcher-selftest.log`, `SELFTEST=1` expecting `stress=9` for the solved
  component): `FAIL: E network: expected 'CREATE_NETWORK … stress=9 …', got '… stress=8 …'`,
  `EXIT:1`.
- **E2E log gate:** `check-e2e-logs.py create-e2e --strict` → `logs=3 ok=3 missing-exit=0 bad-exit=0`.

## Next slice (0.1.2 — press behaviour + cross-mod gate)

**Family:** turn the mechanical press from a stress node into a working consumer, and close the
active cross-mod gate with a paired dependent port.

**Gap line:** coverage table row `mechanical press *pressing* behaviour — not ported — 0.1.2: the
press is a stress node in 0.1.1, but no item transform runs yet`; second gap
`per-tick re-solve in block_entity_tick — blocked(no-level-in-tick-event)`. This is still the
`machines` half of Create's deep tier, so it fits the approved approach; the cross-mod half needs
another port's write ownership and must be dispatched as a paired brief with disjoint owners.

**First surface and concrete assertion:** give the press a one-item input slot and a press recipe
(the artefact's andesite-alloy pressing path), read the solved `Speed` from the adjacent shaft's
published NBT (mechanism 2, not a Rust symbol), and after enough ticks assert through
`ctl screen`/`ctl inventory` that the input was consumed and the stated output appeared; the
counterfactual is the same press with no adjacent source, which must leave the input untouched.
Pair with a dependent add-on slice that reads the shaft's `Speed`/`Ports` and report the gate closed
only if that port's own parsed readout asserts the value.
