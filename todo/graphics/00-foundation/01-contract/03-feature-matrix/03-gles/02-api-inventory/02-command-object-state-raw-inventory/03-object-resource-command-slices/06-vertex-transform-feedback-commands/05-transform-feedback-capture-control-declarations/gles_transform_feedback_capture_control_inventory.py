#!/usr/bin/env python3
"""Validate source-only F03.3.2.2.3.6.5 GLES capture-control declarations."""

from pathlib import Path
import importlib.util

HERE = Path(__file__).resolve().parent
PACKAGE = HERE.parents[10] / "scripts/graphics/inventories"
BOOTSTRAP = PACKAGE / "bootstrap.py"
if BOOTSTRAP.is_symlink() or not BOOTSTRAP.is_file():
    raise ValueError("fixed inventory bootstrap must be a regular file")
spec = importlib.util.spec_from_file_location("inventory_bootstrap", BOOTSTRAP)
bootstrap = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bootstrap)
API = bootstrap.load(PACKAGE, "_vertex_inventory_05")
InventoryEngine, InventoryError = API.InventoryEngine, API.InventoryError
pretty_json, private = API.pretty_json, API.private

ENGINE = InventoryEngine(
    HERE, "gles_transform_feedback_capture_control_catalog.py", "f03322365",
    "webboxvm-gles32-transform-feedback-capture-control-raw-inventory", "transform-feedback",
    "transform-feedback-capture-control-literals-only",
)
CATALOG, GRAMMAR, CACHE = ENGINE.CATALOG, ENGINE.GRAMMAR, ENGINE.CACHE
ARTIFACT, DOMAIN_API, LEDGER = ENGINE.ARTIFACT, ENGINE.DOMAIN_API, ENGINE.LEDGER
INVENTORY = ENGINE.INVENTORY
rendered, validate, records = ENGINE.rendered, ENGINE.validate, ENGINE.records


if __name__ == "__main__":
    ENGINE.main()
