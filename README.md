# Create for Horizon — 0.1.0 (interop surface)

A native `.hmod` for Horizon (pin `0a875da` / mixins `e59da90`, game 26.3). Slice 1 publishes
the world/data surface dependent add-ons bind to:

- one block, `create:creative_motor`, whose block entity publishes the four fields add-ons read
  from a neighbouring Create block — `Speed`, `Capacity`, `Stress` and `NetworkStatus`;
- a parsed command, `/create source <x> <y> <z>`, that returns the four values as
  `CREATE_SOURCE x=… y=… z=… speed=… capacity=… stress=… status=…`;
- the Create ids named by viable add-ons (motor block item, `andesite_alloy`, brass and sheet
  items, one craftable recipe, its loot table and the `c:nuggets/iron` tag), all with regenerated
  art.

There is deliberately **no network solver and no fake solver** in 0.1.0: a standalone source
publishes its own configured speed (16 RPM, the artefact's canonical source value), its declared
capacity (16,384 SU), zero stress and `Generated`/`Idle` status. Connected propagation and real
stress accounting are 0.1.1. There is no cross-mod Rust API; interop is at the world/data level.

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
