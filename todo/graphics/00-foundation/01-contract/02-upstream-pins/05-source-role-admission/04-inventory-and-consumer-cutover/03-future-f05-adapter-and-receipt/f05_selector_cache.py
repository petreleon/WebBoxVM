"""Isolate the sealed selector-cache API for the F05 aggregate receipt."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import f05_source_adapter as adapter

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[7]
SOURCE_CACHE_API = HERE.parent / "01-role-aware-source-contract" / "role_aware_source_cache.py"


class F05SelectorCacheError(ValueError):
    """The F05 receipt cannot use an exact fresh external selector cache."""


def reject(message: str) -> None:
    raise F05SelectorCacheError(message)


def _cache_api():
    names = (*adapter._BARE_MODULES, "role_aware_source_cache")
    before, prior_path = {name: sys.modules.get(name) for name in names}, list(sys.path)
    private, prior_private = "f05_selector_cache_api", sys.modules.get("f05_selector_cache_api")
    try:
        for name in names:
            sys.modules.pop(name, None)
        sys.path.insert(0, str(SOURCE_CACHE_API.parent))
        spec = importlib.util.spec_from_file_location(private, SOURCE_CACHE_API)
        if spec is None or spec.loader is None:
            reject("cannot load the sealed selector-cache API")
        module = importlib.util.module_from_spec(spec)
        sys.modules[private] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != SOURCE_CACHE_API:
            reject("selector-cache API resolved from an unexpected path")
        return module
    except F05SelectorCacheError:
        raise
    except Exception as error:
        reject(f"cannot load the sealed selector-cache API: {error}")
    finally:
        sys.path[:] = prior_path
        for name, module in before.items():
            if module is None:
                sys.modules.pop(name, None)
            else:
                sys.modules[name] = module
        if prior_private is None:
            sys.modules.pop(private, None)
        else:
            sys.modules[private] = prior_private


def external_root(path: Path) -> Path:
    if not isinstance(path, Path) or not path.is_absolute() or path.is_symlink():
        reject("selector cache root must be an absolute nonsymlink path")
    root = path.resolve()
    if root == REPO or REPO in root.parents:
        reject("selector cache root must remain external to the repository")
    return root


def verify_selector_cache(cache_root: Path) -> dict[str, object]:
    root, module = external_root(cache_root), _cache_api()
    try:
        contract = module.source_lock.load_locked_contract()
        verified = module.verify_selector_cache(contract, root)
    except Exception as error:
        reject(f"sealed selector cache cannot be verified: {error}")
    files = [path for path in root.rglob("*") if path.is_file() and not path.is_symlink()]
    if len(files) != 9 or set(verified) != {"source_contract_sha256", "selector_record_ids", "release_proof_ids"}:
        reject("selector cache does not contain the exact nine verified files")
    return {"mode": "fresh-refresh", "file_count": 9, **verified}


def refresh_selector_cache(cache_root: Path, timeout: float) -> dict[str, object]:
    root, module = external_root(cache_root), _cache_api()
    try:
        contract = module.source_lock.load_locked_contract()
        module.refresh_root_selectors(contract, root, timeout)
    except Exception as error:
        reject(f"fresh selector cache cannot be captured: {error}")
    return verify_selector_cache(root)
