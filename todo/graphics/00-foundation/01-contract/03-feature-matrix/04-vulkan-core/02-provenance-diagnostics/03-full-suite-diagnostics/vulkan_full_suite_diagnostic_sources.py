"""Fixed-path source loaders for F03.4.2.3's root-wide VCTS diagnostic."""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[7]
BINDINGS = HERE.parents[2] / "01-profile-scope/role_aware_bindings.py"
CHANNEL = HERE.parent / "01-source-channel-boundary/vulkan_source_channels.py"
LEDGER = (HERE.parents[3] / "02-upstream-pins/05-source-role-admission/03-full-suite-record/"
          "03-vulkan-default-full-ledger/01-vulkan-ledger-taxonomy/vulkan_ledger_taxonomy.py")
CACHE = (HERE.parents[3] / "02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/"
         "01-role-aware-source-contract/role_aware_source_cache.py")
BARE = ("role_aware_bindings", "role_aware_source_lock", "role_aware_source_contract",
        "role_aware_source_evidence", "full_suite_roots", "normative_roots",
        "vulkan_definition_contract", "source_role_contract", "source_role_records",
        "source_role_artifacts", "source_cache", "source_model", "normative_notices",
        "webboxvm_source_builder", "inventory_layout", "f02531_identity", "f02531_ledger",
        "f02531_taxonomy", "registry_inventory_source", "registry_inventory_contract",
        "registry_inventory_validation")


class DiagnosticSourceError(ValueError):
    """A required fixed VCTS source cannot prove a root-wide observation."""


def reject(message: str) -> None:
    raise DiagnosticSourceError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            reject("diagnostic has duplicate JSON fields")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"diagnostic cannot be read: {error}")
    if not isinstance(value, dict):
        reject("diagnostic is not a JSON object")
    return value


def private(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        reject("fixed source API must be a regular file")
    old, prior, old_path = {item: sys.modules.get(item) for item in BARE}, sys.modules.get(name), list(sys.path)
    try:
        for item in BARE:
            sys.modules.pop(item, None)
        sys.path.insert(0, str(path.parent))
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            reject("cannot load fixed source API")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            reject("fixed source API resolved from an unexpected path")
        return module
    except DiagnosticSourceError:
        raise
    except Exception as error:
        reject(f"cannot load fixed source API: {error}")
    finally:
        sys.path[:] = old_path
        for item, module in old.items():
            if module is None:
                sys.modules.pop(item, None)
            else:
                sys.modules[item] = module
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


def bindings_api():
    return private(BINDINGS, "f03423_role_aware_bindings")


def channel_api():
    return private(CHANNEL, "f03423_vulkan_source_channels")


def ledger_api():
    return private(LEDGER, "f03423_vulkan_ledger_taxonomy")


def cache_api():
    return private(CACHE, "f03423_role_aware_source_cache")
