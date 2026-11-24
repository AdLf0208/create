# Create for Horizon — port notes (slice 1, `0.1.0`)

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

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

**Carried set (exact):**
- block: `create:creative_motor` — the interop surface; its block entity publishes the four fields.
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
| BE readout `Speed`/`Capacity`/`Stress`/`NetworkStatus` | tested | parsed `/create source 300 101 0` → exact `CREATE_SOURCE … speed=16 capacity=16384 stress=0 status=Running`; vanilla block refusal; no-mod control `EXIT:1`; `SELFTEST=1` `EXIT:1` |
| regenerated art / monogram icon / MIT+NOTICE | tested | `check_provenance.py --check`: 0 matches; `gen_assets.py --check` reproducible; LICENSE + NOTICE packed |
| propagation / stress accounting / machines | not ported | 0.1.1 kinetics engine |
| Ponder / trains / contraptions / schematics | not ported | out of scope (deep tier) |

## Upstream defects fixed

None found in slice 1. The creative motor's RPM is GUI-configured and has no blockstate default; the
port fixes the standalone source at the artefact's own canonical 16 RPM and records that choice above.

## Verification (re-read from the artefacts)

- **Delivered package:** `create.hmod`, **23,894 B**, sha256
  `2272818e7b35038e787786b2c72bcbea399ec8761b9ff928be9c7c3277dc2cf7`, **48 entries**, no
  `Cargo.lock`. `cargo horizon-mod check`: `native-mod, format native-1/0a875da…/e59da90…, for 26.3`.
- **Red, verbatim** (`ports/_shared/evidence/create/sdk-test-red.log`, the 4 tests before
  `source_state` existed):
  `error[E0425]: cannot find function `source_state` in this scope` (7 occurrences), ending
  `error: could not compile `create` (lib test) due to 7 previous errors` — exit 101.
- **Green tests / lints** (`sdk-test.log`, `final-clippy.log`, `final-fmt.log`): 4 passed / 0
  failed; clippy `--all-targets -- -D warnings` exit 0; `cargo fmt --check` exit 0.
- **Both `fits` lines on the delivered sha** (`sdk-fits.log`, `sdk-fits-other.log`):
  `0a875da  made for` (exit 0) and `0b14f8a  fits` (exit 0).
- **Zero jar-derived asset bytes** (`check_provenance.py --check`):
  `OK: zero jar-derived asset bytes — 39 candidate files, 48 packaged files, 0 matches against
  11548 jar entries`. `PROVENANCE.md` is the per-file report; `gen_assets.py --check` re-renders
  all 39 generated files byte-for-byte.
- **SDK gate:** `verify-sdk-package.py create` → **1/1** (package sha, `runtime.rs` byte-identical,
  `package matches the tree`, R9 tables/deps/build-script, no lock, tests, clippy, check, fits,
  skills gate).
- **Dialect:** `check_261_dialect.py ports/create` → both recipe and loot table clean.
- **In-game transcript** (green `e2e-launcher.log`, launcher-built game at the pin):
  `CREATE_SOURCE x=300 y=101 z=0 speed=16 capacity=16384 stress=0 status=Running` (phase A);
  `CREATE_SOURCE refused: no Create block entity at [304, 101, 0]` (phase B, vanilla stone);
  `Gave 2 [Andesite Alloy]` → `ctl inventory` count 2 (C); grid result `create:andesite_alloy`
  with `{"create:andesite_alloy":1, "minecraft:andesite":2, "minecraft:iron_nugget":2}` (D);
  `PASS … EXIT:0`.
- **Control** (`e2e-launcher-control.log`, unmodded Nightly): `Unknown block type
  'create:creative_motor'`, `Unknown or incomplete command … create source …`, `FAIL … EXIT:1`.
- **Selftest** (`e2e-launcher-selftest.log`, `SELFTEST=1` expecting `speed=999`):
  `FAIL: A readout … got speed=16 … EXIT:1`.
- **E2E log gate:** `check-e2e-logs.py create-e2e --strict` → `logs=3 ok=3 missing-exit=0 bad-exit=0`.

## Next slice (0.1.1 — kinetics engine)

**Family:** the pure network solver and the first power-transmission block. Implement the connected
kinetics model in `runtime.rs` (block-id → node recognition for `create:shaft`, `cogwheel`,
`large_cogwheel`, `encased_shaft` and the motor; neighbour propagation; per-node rotation; stress
accounting `Σ impact × |rpm|`; status `Running`/`Overstressed`/`Idle`), then carry the `shaft` block
and one consumer (`mechanical_press`, impact 8 SU) and publish the solved values from every node's
tick. This fits the tier: it is the `machines` half of the deep-tier core the add-ons need, and it
replaces slice 1's documented "no solver" limitation.

**Gap line:** coverage table row `propagation / stress accounting / machines — not ported — 0.1.1
kinetics engine`.

**In-game assertion:** place `create:creative_motor` at (300,101,0), a `create:shaft[axis=x]` at
(301,101,0) and a `create:mechanical_press` at (302,101,0); through `ctl chat` read
`/create network 301 101 0` and assert the **parsed** line
`CREATE_NETWORK x=301 y=101 z=0 speed=16 stress=8 capacity=16384 status=Running`, with a control
shaft at (305,101,0) (not adjacent to any source) answering `speed=0 … status=Idle` and a
`SELFTEST` expecting `stress=9` ending `EXIT:1`. Keep the differential: a press the solver does not
see must not contribute stress.
