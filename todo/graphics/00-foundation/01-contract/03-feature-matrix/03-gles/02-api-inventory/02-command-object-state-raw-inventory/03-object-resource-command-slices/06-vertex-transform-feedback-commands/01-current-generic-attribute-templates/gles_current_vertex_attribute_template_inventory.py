#!/usr/bin/env python3
"""Bounded GLES source inventory f03322361; shared admission and row validation."""

import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent
PACKAGE = HERE.parents[10] / "scripts/graphics/inventories"
BOOTSTRAP = PACKAGE / "bootstrap.py"
if BOOTSTRAP.is_symlink() or not BOOTSTRAP.is_file():
    raise ValueError("fixed inventory bootstrap must be a regular file")
spec = importlib.util.spec_from_file_location("inventory_bootstrap", BOOTSTRAP)
bootstrap = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bootstrap)
API = bootstrap.load(PACKAGE, "_vertex_inventory_01")
InventoryEngine, InventoryError = API.InventoryEngine, API.InventoryError
pretty_json, private = API.pretty_json, API.private

ENGINE = InventoryEngine(
    HERE, 'gles_current_vertex_attribute_template_catalog.py', 'f03322361',
    'webboxvm-gles32-current-vertex-attribute-template-raw-inventory', 'vertex-array',
    'current-generic-attribute-templates-only', 'template-expansion', 'gles_current_vertex_attribute_template_inventory.json',
)
CATALOG, ARTIFACT, DOMAIN_API = ENGINE.CATALOG, ENGINE.ARTIFACT, ENGINE.DOMAIN_API
GRAMMAR, CACHE, LEDGER, INVENTORY = ENGINE.GRAMMAR, ENGINE.CACHE, ENGINE.LEDGER, ENGINE.INVENTORY
canonical, exact = ENGINE.canonical, ENGINE.exact
rendered, validate, records = ENGINE.rendered, ENGINE.validate, ENGINE.records
source_input, reject_unadmitted = ENGINE.source_input, ENGINE.reject_unadmitted

if __name__ == "__main__":
    ENGINE.main()
