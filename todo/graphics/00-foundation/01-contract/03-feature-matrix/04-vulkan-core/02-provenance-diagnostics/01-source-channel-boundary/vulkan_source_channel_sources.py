"""Fixed-path loaders shared by the F03.4.2.1 channel boundary."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BINDINGS = HERE.parents[2] / "01-profile-scope/role_aware_bindings.py"
INVENTORY = HERE.parents[1] / "01-registry-inventory/v2/registry_inventory_validation.py"


class BoundaryError(ValueError):
    """A Vulkan source channel escapes its admitted role or no-claim limit."""


def reject(message: str) -> None:
    raise BoundaryError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            reject("boundary has duplicate JSON fields")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"boundary cannot be read: {error}")
    if not isinstance(value, dict):
        reject("boundary is not a JSON object")
    return value


def private(path: Path, name: str, bare=()):
    if path.is_symlink() or not path.is_file():
        reject("fixed source API must be a regular file")
    old, prior, old_path = {item: sys.modules.get(item) for item in bare}, sys.modules.get(name), list(sys.path)
    try:
        for item in bare:
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
    except BoundaryError:
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
    return private(BINDINGS, "f03421_role_aware_bindings")


def inventory_api():
    return private(INVENTORY, "f03421_registry_validation",
                   ("registry_inventory_source", "registry_inventory_contract", "registry_inventory_validation"))
