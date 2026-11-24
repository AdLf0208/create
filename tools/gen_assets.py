#!/usr/bin/env python3
"""Generate every carried asset for the Create port — the port's own bytes.

Create's code is MIT, but `src/main/resources/assets/` in the cached jar is **All
Rights Reserved** (verified from Creators-of-Create/Create LICENSE.md). Not one
upstream texture, model, lang file or icon is read or copied here: only the **ids**
and the **shapes** are followed, and every pixel comes from the palette below.
The data files are re-authored in the 26.3 dialect (bare-string recipe ingredients,
one `condition` object with `type`, integral `rolls`) from the jar's semantics.

Output is deterministic: PNGs use stored (uncompressed) deflate blocks, so the same
palette always gives the same bytes. `--check` re-renders into memory and compares.

    python3 tools/gen_assets.py            # write assets/, data/ and icon.png
    python3 tools/gen_assets.py --check    # exit 1 if any file differs

Palette (one named colour per family):

    BRASS        #d0a24a   creative motor shell, brass ingot/nugget/sheet
    ANDESITE     #8d8b86   andesite alloy
    COPPER       #c1703c   copper sheet
    IRON         #cfd3d6   iron sheet
    DARK         #2b2b30   motor recess / icon ground
    ACCENT       #66d0c4   motor's lit panel and the monogram's highlight
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import struct
import sys
import zlib

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets" / "create"
DATA = ROOT / "data"
ICON = ROOT / "icon.png"

PALETTE = {
    "brass": (0xD0, 0xA2, 0x4A),
    "andesite": (0x8D, 0x8B, 0x86),
    "copper": (0xC1, 0x70, 0x3C),
    "iron": (0xCF, 0xD3, 0xD6),
    "dark": (0x2B, 0x2B, 0x30),
    "accent": (0x66, 0xD0, 0xC4),
}
TRANSPARENT = (0, 0, 0, 0)

ITEM_IDS = [
    "creative_motor",
    "andesite_alloy",
    "brass_ingot",
    "brass_nugget",
    "iron_sheet",
    "brass_sheet",
    "copper_sheet",
]
ITEM_COLOUR = {
    "creative_motor": "brass",
    "andesite_alloy": "andesite",
    "brass_ingot": "brass",
    "brass_nugget": "brass",
    "iron_sheet": "iron",
    "brass_sheet": "brass",
    "copper_sheet": "copper",
}


def rgba(name: str) -> tuple[int, int, int, int]:
    r, g, b = PALETTE[name]
    return (r, g, b, 255)


def png(width: int, height: int, pixels: list[tuple[int, int, int, int]]) -> bytes:
    """An RGBA PNG with stored deflate blocks: same input, same bytes on any Python."""
    raw = bytearray()
    for y in range(height):
        raw.append(0)  # filter: none
        for x in range(width):
            raw.extend(pixels[y * width + x])

    def chunk(kind: bytes, payload: bytes) -> bytes:
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
        )

    deflate = bytearray(b"\x78\x01")
    data = bytes(raw)
    for start in range(0, len(data), 0xFFFF):
        block = data[start : start + 0xFFFF]
        final = 1 if start + 0xFFFF >= len(data) else 0
        deflate.append(final)
        deflate.extend(struct.pack("<HH", len(block), len(block) ^ 0xFFFF))
        deflate.extend(block)
    deflate.extend(struct.pack(">I", zlib.adler32(data) & 0xFFFFFFFF))
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", bytes(deflate))
        + chunk(b"IEND", b"")
    )


def motor_block_texture() -> bytes:
    """Brass shell with a dark recess and a lit teal panel."""
    pixels = []
    for y in range(16):
        for x in range(16):
            if 4 <= x < 12 and 4 <= y < 12:
                colour = rgba("dark")
            elif 6 <= x < 10 and 6 <= y < 10:
                colour = rgba("accent")
            else:
                colour = rgba("brass")
            pixels.append(colour)
    return png(16, 16, pixels)


def item_texture(kind: str) -> bytes:
    """A 16x16 silhouette in the item's family colour: ingot, nugget, sheet or motor."""
    name = ITEM_COLOUR[kind]
    pixels = []
    for y in range(16):
        for x in range(16):
            inside = False
            if kind.endswith("_ingot"):
                inside = 3 <= x < 13 and 6 <= y < 10
            elif kind.endswith("_nugget"):
                inside = 5 <= x < 11 and 5 <= y < 11
            elif kind.endswith("_sheet"):
                inside = 2 <= x < 14 and 2 <= y < 14
            elif kind == "creative_motor":
                inside = 3 <= x < 13 and 3 <= y < 13 and not (6 <= x < 10 and 6 <= y < 10)
            else:  # andesite_alloy
                inside = 4 <= x < 12 and 4 <= y < 12
            pixels.append(rgba(name) if inside else TRANSPARENT)
    return png(16, 16, pixels)


def monogram_icon() -> bytes:
    """A 64x64 monogram: a brass 'C' with a teal accent on a dark ground."""
    pixels = []
    for y in range(64):
        for x in range(64):
            on_ring = (
                12 <= x < 52
                and 12 <= y < 52
                and (
                    (18 <= x < 24 and 18 <= y < 46)
                    or (18 <= x < 46 and 18 <= y < 24)
                    or (18 <= x < 46 and 40 <= y < 46)
                    or (40 <= x < 46 and 18 <= y < 40)
                )
            )
            if on_ring:
                colour = rgba("accent") if 28 <= y < 36 and 40 <= x < 46 else rgba("brass")
            else:
                colour = rgba("dark")
            pixels.append(colour)
    return png(64, 64, pixels)


def blockstate() -> str:
    variants = {}
    for facing, (model, rot) in {
        "south": ("block", 0),
        "west": ("block", 90),
        "north": ("block", 180),
        "east": ("block", 270),
        "up": ("block_vertical", 0),
        "down": ("block_vertical", 180),
    }.items():
        entry: dict[str, object] = {"model": f"create:block/creative_motor/{model}"}
        if rot:
            entry["y" if model == "block" else "x"] = rot
        variants[f"facing={facing}"] = entry
    return json.dumps({"variants": variants}, indent=2) + "\n"


def block_models() -> dict[str, str]:
    base = {
        "parent": "block/block",
        "textures": {
            "particle": "create:block/creative_motor",
            "texture": "create:block/creative_motor",
        },
        "elements": [
            {
                "from": [1, 1, 1],
                "to": [15, 15, 15],
                "faces": {
                    f: {"texture": "#texture", "cullface": f}
                    for f in ("down", "up", "north", "south", "west", "east")
                },
            }
        ],
    }
    return {
        "block": json.dumps(base, indent=2) + "\n",
        "block_vertical": json.dumps(base, indent=2) + "\n",
        "item": json.dumps({"parent": "create:block/creative_motor/block"}, indent=2) + "\n",
    }


def item_model(kind: str) -> str:
    if kind == "creative_motor":
        return json.dumps({"parent": "create:block/creative_motor/item"}, indent=2) + "\n"
    return json.dumps(
        {
            "parent": "minecraft:item/generated",
            "textures": {"layer0": f"create:item/{kind}"},
        },
        indent=2,
    ) + "\n"


def render() -> dict[str, bytes | str]:
    out: dict[str, bytes | str] = {}
    out["assets/create/textures/block/creative_motor.png"] = motor_block_texture()
    for kind in ITEM_IDS:
        out[f"assets/create/textures/item/{kind}.png"] = item_texture(kind)
    out["icon.png"] = monogram_icon()

    out["assets/create/blockstates/creative_motor.json"] = blockstate()
    for name, body in block_models().items():
        out[f"assets/create/models/block/creative_motor/{name}.json"] = body
    for kind in ITEM_IDS:
        out[f"assets/create/models/item/{kind}.json"] = item_model(kind)
        out[f"assets/create/items/{kind}.json"] = json.dumps(
            {"model": {"type": "minecraft:model", "model": f"create:item/{kind}"}}, indent=2
        ) + "\n"

    names = {
        "creative_motor": "Creative Motor",
        "andesite_alloy": "Andesite Alloy",
        "brass_ingot": "Brass Ingot",
        "brass_nugget": "Brass Nugget",
        "iron_sheet": "Iron Sheet",
        "brass_sheet": "Brass Sheet",
        "copper_sheet": "Copper Sheet",
    }
    lang = {f"block.create.{k}": v for k, v in names.items()}
    lang.update({f"item.create.{k}": v for k, v in names.items()})
    lang["itemGroup.create.create"] = "Create"
    out["assets/create/lang/en_us.json"] = json.dumps(lang, indent=2, sort_keys=True) + "\n"

    out["data/create/horizon/block/creative_motor.json"] = json.dumps(
        {
            "template": "minecraft:stone",
            "item": True,
            "properties": {
                "explosion_resistance": 3.0,
                "hardness": 3.0,
                "occludes": False,
                "render_shape": "MODEL",
                "requires_tool": False,
                "sounds": "minecraft:stone",
            },
            "state_properties": {
                "facing": {
                    "type": "enum",
                    "values": ["down", "up", "north", "south", "west", "east"],
                }
            },
            "default_state": {"facing": "south"},
        },
        indent=2,
    ) + "\n"
    out["data/create/loot_table/blocks/creative_motor.json"] = json.dumps(
        {
            "type": "minecraft:block",
            "random_sequence": "create:blocks/creative_motor",
            "pools": [
                {
                    "rolls": 1,
                    "condition": {"type": "minecraft:survives_explosion"},
                    "entries": [{"type": "minecraft:item", "name": "create:creative_motor"}],
                }
            ],
        },
        indent=2,
    ) + "\n"
    out["data/create/recipe/andesite_alloy.json"] = json.dumps(
        {
            "type": "minecraft:crafting_shaped",
            "category": "misc",
            "group": "create:andesite_alloy",
            "key": {"A": "minecraft:andesite", "B": "#c:nuggets/iron"},
            "pattern": ["BA", "AB"],
            "result": {"count": 1, "id": "create:andesite_alloy"},
        },
        indent=2,
    ) + "\n"
    out["data/c/tags/item/nuggets/iron.json"] = json.dumps(
        {"values": ["minecraft:iron_nugget"]}, indent=2
    ) + "\n"
    for kind in ITEM_IDS:
        if kind == "creative_motor":
            continue  # the block declaration creates its block item
        out[f"data/create/horizon/item/{kind}.json"] = json.dumps(
            {"template": "minecraft:iron_ingot"}, indent=2
        ) + "\n"
    out["data/create/horizon/creative_tab/create.json"] = json.dumps(
        {
            "title": "itemGroup.create.create",
            "icon": "create:creative_motor",
            "items": [f"create:{kind}" for kind in ITEM_IDS],
        },
        indent=2,
    ) + "\n"
    return out


def write_files(files: dict[str, bytes | str]) -> None:
    for rel, body in files.items():
        path = ROOT / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(body if isinstance(body, bytes) else body.encode("utf-8"))


def main() -> int:
    files = render()
    if "--check" in sys.argv:
        bad = []
        for rel, body in files.items():
            path = ROOT / rel
            want = body if isinstance(body, bytes) else body.encode("utf-8")
            if not path.is_file() or path.read_bytes() != want:
                bad.append(rel)
        if bad:
            for rel in bad:
                print(f"FAIL: {rel} differs from the generated bytes")
            return 1
        print(f"OK: {len(files)} generated files match byte for byte")
        return 0
    write_files(files)
    print(f"wrote {len(files)} generated files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
