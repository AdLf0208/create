# Create for Horizon — 0.1.1 (kinetics engine + interop surface)

A native `.hmod` for Horizon (pin `0a875da` / mixins `e59da90`, game 26.3). It publishes the
world/data surface dependent add-ons bind to and, from 0.1.1, solves connected rotation:

- `create:creative_motor`, `create:shaft`, `create:cogwheel`, `create:large_cogwheel`,
  `create:encased_shaft` and `create:mechanical_press` each carry a block entity publishing
  `Speed`, `Capacity`, `Stress` and `NetworkStatus` in its saved NBT;
- a pure, bounded network solver in `src/runtime.rs` recognises those ids, propagates rotation
  across connected axes, and charges stress as `Σ impact × |speed| / 16` (a press is 8 SU at the
  artefact's canonical 16 RPM);
- parsed commands `/create source <x> <y> <z>` and `/create network <x> <y> <z>` return the four
  values, and the network command publishes the result into every node's saved data — the same
  read path an add-on uses;
- the Create ids named by viable add-ons (motor block item, `andesite_alloy`, brass and sheet
  items, one craftable recipe, its loot table and the `c:nuggets/iron` tag), all with regenerated
  art.

There is no cross-mod Rust API: interop is at the world/data level only. Belts, chains, clutches,
contraptions, trains and Ponder are out of scope; see `notes.md` for the stated simplifications.

## Building and checking

```sh
python3 tools/gen_assets.py --check          # art is reproducible
python3 tools/inventory.py --check           # the carried-id contract
python3 tools/check_provenance.py --check    # zero jar-derived asset bytes
ports/tools/verify-sdk-package.py create     # the package gate
```

Every build/test/e2e step goes through `ports/tools/build-slot.sh` with `--port ports/create`.

## Licence

MIT code (The Create Team / contributors); the All-Rights-Reserved upstream assets are **not**
carried — see [`LICENSE`](LICENSE), [`LICENSE-NOTICE.md`](LICENSE-NOTICE.md) and
[`PROVENANCE.md`](PROVENANCE.md).
