"""Fixed-path F02/F03 provenance helpers for F05 profile registration."""

from __future__ import annotations

import copy
import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
FOUNDATION = HERE.parents[2]
SOURCE_ADAPTER = FOUNDATION / "01-contract/02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/03-future-f05-adapter-and-receipt/f05_source_adapter.py"
REGISTRY_DIR = FOUNDATION / "01-contract/03-feature-matrix/04-vulkan-core/01-registry-inventory/v2"
REGISTRY_CACHE = REGISTRY_DIR / "registry_inventory_cache.py"
REGISTRY_BARE = ("registry_inventory_source", "registry_inventory_cache")


class SourceError(ValueError):
    """A profile registration lacks its exact sealed source boundary."""


def reject(message: str) -> None:
    raise SourceError(message)


def private_module(path: Path, name: str, directory: Path, bare: tuple[str, ...] = ()):  # noqa: ANN201
    if path.is_symlink() or not path.is_file():
        reject(f"fixed source API is unavailable: {path.name}")
    before, old_path = {item: sys.modules.get(item) for item in bare}, list(sys.path)
    prior = sys.modules.get(name)
    try:
        for item in bare:
            sys.modules.pop(item, None)
        sys.path.insert(0, str(directory))
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            reject(f"cannot load fixed source API: {path.name}")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            reject(f"fixed source API resolved from an unexpected path: {path.name}")
        return module
    except SourceError:
        raise
    except Exception as error:
        reject(f"cannot load fixed source API {path.name}: {error}")
    finally:
        sys.path[:] = old_path
        for item, module in before.items():
            if module is None:
                sys.modules.pop(item, None)
            else:
                sys.modules[item] = module
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


def admitted_contract() -> dict[str, object]:
    module = private_module(SOURCE_ADAPTER, "f05_profile_source_adapter", SOURCE_ADAPTER.parent)
    try:
        value = module.admitted_source_contract()
    except Exception as error:
        reject(f"sealed F02 source contract is unavailable: {error}")
    if not isinstance(value, dict):
        reject("sealed F02 source contract is malformed")
    return copy.deepcopy(value)


def verify_registry_cache(path: Path, source_contract_sha256: str) -> dict[str, object]:
    module = private_module(REGISTRY_CACHE, "f05_profile_registry_cache", REGISTRY_DIR, REGISTRY_BARE)
    try:
        value = module.verify_selector_cache(path)
    except Exception as error:
        reject(f"external Vulkan selector cache is unavailable: {error}")
    if (not isinstance(value, dict) or value.get("source_contract_sha256") != source_contract_sha256
            or "vulkan-registry" not in value.get("selector_record_ids", [])):
        reject("external Vulkan selector cache has a stale or incomplete source contract")
    return copy.deepcopy(value)
