#!/usr/bin/env python3
"""Check the carried-id contract against the port's content tree.

`--check` fails on an empty tree and passes only when every id in
`tools/carry_contract.json` has its file in `data/` or `assets/` and every
counted family matches. This is Task 1's gate: the exact set is decided in
`notes.md` before any art or code is generated.

    python3 tools/inventory.py --check    # gate
    python3 tools/inventory.py            # print the tree summary
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

PORT = Path(__file__).resolve().parent.parent
CONTRACT = PORT / "tools" / "carry_contract.json"


def names(folder: Path, suffix: str = ".json") -> set[str]:
    if not folder.is_dir():
        return set()
    return {p.name[: -len(suffix)] for p in folder.rglob(f"*{suffix}")}


def check() -> int:
    contract = json.loads(CONTRACT.read_text())
    tree = [p for p in (PORT / "assets").rglob("*") if p.is_file()] + [
        p for p in (PORT / "data").rglob("*") if p.is_file()
    ]
    if not tree:
        print("FAIL: the carried tree (assets/, data/) is empty")
        return 1

    errors: list[str] = []
    found = {
        "block_declarations": names(PORT / "data/create/horizon/block"),
        "loot_tables": names(PORT / "data/create/loot_table/blocks"),
        "recipes": names(PORT / "data/create/recipe"),
        "item_tags": names(PORT / "data/c/tags/item"),
        "blockstates": names(PORT / "assets/create/blockstates"),
        "block_models": {
            str(p.relative_to(PORT / "assets/create/models/block"))
            for p in (PORT / "assets/create/models/block").rglob("*.json")
        },
        "item_models": names(PORT / "assets/create/models/item"),
        "generated_textures": {
            str(p.relative_to(PORT / "assets/create/textures")) for p in (PORT / "assets/create/textures").rglob("*.png")
        },
        "lang_files": names(PORT / "assets/create/lang"),
    }
    for family, want in contract["counts"].items():
        got = len(found.get(family, set()))
        if got != want:
            errors.append(f"{family}: expected {want}, found {got}")

    def has(family: str, ident: str) -> None:
        path = ident.split(":", 1)[1]
        if path not in found.get(family, set()):
            errors.append(f"missing {family} id {ident}")

    for ident in contract["blocks"]:
        has("block_declarations", ident)
    for ident in contract["items"]:
        has("item_models", ident)
    for ident in contract["recipes"]:
        has("recipes", ident)
    for ident in contract["loot_tables"]:
        has("loot_tables", ident)
    for ident in contract["tags"]:
        if ident.replace("c:", "") != "nuggets/iron":
            errors.append(f"unexpected tag id {ident}")
        if not (PORT / "data/c/tags/item/nuggets/iron.json").is_file():
            errors.append(f"missing tag file for {ident}")

    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        return 1
    print(
        f"OK: carried set matches the contract — "
        f"{len(contract['blocks'])} block, {len(contract['items'])} items, "
        f"{len(contract['recipes'])} recipe, {len(contract['loot_tables'])} loot table, "
        f"{len(contract['tags'])} tag; {len(tree)} files in the tree"
    )
    return 0


def main() -> int:
    if "--check" in sys.argv:
        return check()
    contract = json.loads(CONTRACT.read_text())
    print(json.dumps(contract, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
