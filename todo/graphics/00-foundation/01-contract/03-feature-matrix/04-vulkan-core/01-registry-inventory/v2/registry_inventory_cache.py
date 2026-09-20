#!/usr/bin/env python3
"""Verify an external complete selector cache before reading the auxiliary registry."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import registry_inventory_source as source

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[7]
SOURCE_CACHE_API = HERE.parents[3] / "02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/role_aware_source_cache.py"
BARE_MODULES = ("role_aware_source_lock", "role_aware_source_contract", "role_aware_source_evidence", "full_suite_roots",
                "normative_roots", "vulkan_definition_contract", "source_role_contract", "source_role_records",
                "source_role_artifacts", "source_cache", "source_model", "normative_notices", "webboxvm_source_builder",
                "inventory_layout", "role_aware_source_cache")


class SelectorCacheError(ValueError):
    """The inventory cannot use an incomplete or non-external selector cache."""


def reject(message: str) -> None:
    raise SelectorCacheError(message)


def external_root(path: Path) -> Path:
    if not isinstance(path, Path) or not path.is_absolute() or path.is_symlink():
        reject("selector cache root must be an absolute nonsymlink path")
    root = path.resolve()
    if root == REPO or REPO in root.parents:
        reject("selector cache root must remain external to the repository")
    return root


def _cache_api():
    if SOURCE_CACHE_API.is_symlink() or not SOURCE_CACHE_API.is_file():
        reject("sealed selector-cache API must be a regular fixed-path file")
    before, previous_path = {name: sys.modules.get(name) for name in BARE_MODULES}, list(sys.path)
    private, prior_private = "f0341_selector_cache_api", sys.modules.get("f0341_selector_cache_api")
    try:
        for name in BARE_MODULES:
            sys.modules.pop(name, None)
        sys.path.insert(0, str(SOURCE_CACHE_API.parent))
        spec = importlib.util.spec_from_file_location(private, SOURCE_CACHE_API)
        if spec is None or spec.loader is None:
            reject("cannot load the sealed selector-cache API")
        module = importlib.util.module_from_spec(spec)
        sys.modules[private] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != SOURCE_CACHE_API.resolve():
            reject("selector-cache API resolved from an unexpected path")
        return module
    except SelectorCacheError:
        raise
    except Exception as error:
        reject(f"cannot load the sealed selector-cache API: {error}")
    finally:
        sys.path[:] = previous_path
        for name, module in before.items():
            if module is None:
                sys.modules.pop(name, None)
            else:
                sys.modules[name] = module
        if prior_private is None:
            sys.modules.pop(private, None)
        else:
            sys.modules[private] = prior_private


def verify_selector_cache(cache_root: Path) -> dict[str, object]:
    root, module = external_root(cache_root), _cache_api()
    try:
        contract = module.source_lock.load_locked_contract()
        verified = module.verify_selector_cache(contract, root)
    except Exception as error:
        reject(f"sealed selector cache cannot be verified: {error}")
    files = [path for path in root.rglob("*") if path.is_file() and not path.is_symlink()]
    fields = {"source_contract_sha256", "selector_record_ids", "release_proof_ids"}
    if len(files) != 9 or not isinstance(verified, dict) or set(verified) != fields:
        reject("selector cache does not contain the exact nine verified files")
    return {"mode": "fresh-refresh", "file_count": 9, **verified}


def payload_from_selector_cache(cache_root: Path, identity: object) -> bytes:
    checked, verified = source.checked_identity(identity), verify_selector_cache(cache_root)
    if verified["source_contract_sha256"] != checked["source_contract_sha256"]:
        reject("selector cache and registry identity use different sealed contracts")
    return source.payload_bytes(external_root(cache_root) / str(checked["artifact"]), checked)
