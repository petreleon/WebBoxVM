"""Shared-engine entry point for the initial literal-trigger lifecycle group."""

import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent
PACKAGE = HERE.parents[9] / "scripts/graphics/inventories"
BOOTSTRAP = PACKAGE / "bootstrap.py"
if BOOTSTRAP.is_symlink() or not BOOTSTRAP.is_file():
    raise ValueError("fixed inventory bootstrap must be a regular file")
SPEC = importlib.util.spec_from_file_location("lifecycle_inventory_bootstrap", BOOTSTRAP)
if SPEC is None or SPEC.loader is None:
    raise ValueError("cannot load fixed inventory bootstrap")
BOOT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BOOT)
API = BOOT.load(PACKAGE, "_state_inventory_01")
InventoryEngine, InventoryError = API.InventoryEngine, API.InventoryError
pretty_json, private = API.pretty_json, API.private
ENGINE = InventoryEngine(HERE, "gles_context_state_lifecycle_catalog.py", "f0332241",
                         "webboxvm-gles32-context-state-lifecycle-raw-inventory", "context-state-lifecycle",
                         "context-state-lifecycle-triggered-rules-only", "explicit-triggered-rule",
                         raw_root=HERE.parents[1])
CATALOG, ARTIFACT, DOMAIN_API, GRAMMAR = ENGINE.CATALOG, ENGINE.ARTIFACT, ENGINE.DOMAIN_API, ENGINE.GRAMMAR
CACHE, LEDGER, INVENTORY = ENGINE.CACHE, ENGINE.LEDGER, ENGINE.INVENTORY
rendered, validate, records = ENGINE.rendered, ENGINE.validate, ENGINE.records

if __name__ == "__main__":
    ENGINE.main()
