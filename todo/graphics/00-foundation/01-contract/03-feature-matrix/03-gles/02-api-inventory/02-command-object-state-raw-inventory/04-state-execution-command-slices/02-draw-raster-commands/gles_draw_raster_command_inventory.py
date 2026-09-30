#!/usr/bin/env python3
"""Validate complete source-only F03.3.2.2.4.2 formal declarations."""

import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent
PACKAGE = HERE.parents[9] / "scripts/graphics/inventories"
BOOTSTRAP = PACKAGE / "bootstrap.py"
if BOOTSTRAP.is_symlink() or not BOOTSTRAP.is_file():
    raise ValueError("fixed inventory bootstrap must be a regular file")
spec = importlib.util.spec_from_file_location("inventory_bootstrap", BOOTSTRAP)
bootstrap = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bootstrap)
API = bootstrap.load(PACKAGE, "_state_inventory_02")
InventoryEngine, InventoryError = API.InventoryEngine, API.InventoryError
pretty_json, private = API.pretty_json, API.private
ENGINE = InventoryEngine(
    HERE, "gles_draw_raster_command_catalog.py", "f0332242",
    "webboxvm-gles32-draw-raster-raw-inventory", "draw-raster",
    "formal-declarations-only-semantic-routes-excluded", derivation_class="formal-declaration",
    raw_root=HERE.parents[1],
)
CATALOG, GRAMMAR, CACHE = ENGINE.CATALOG, ENGINE.GRAMMAR, ENGINE.CACHE
ARTIFACT, DOMAIN_API, LEDGER = ENGINE.ARTIFACT, ENGINE.DOMAIN_API, ENGINE.LEDGER
INVENTORY = ENGINE.INVENTORY
rendered, validate, records = ENGINE.rendered, ENGINE.validate, ENGINE.records

if __name__ == "__main__":
    ENGINE.main()
