#!/usr/bin/env python3
"""Atomically refresh the seven small root selectors under one sealed source contract."""

from __future__ import annotations

import argparse
import os
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE_ROLE = HERE.parents[1]
ROLE = SOURCE_ROLE / "01-authority-and-transform-boundary"
NORMATIVE = SOURCE_ROLE / "02-normative-build-record/01-normative-root-pins"
SUITES = SOURCE_ROLE / "03-full-suite-record/01-canonical-full-suite-roots"
FETCH = SOURCE_ROLE.parent / "02-fetch-verifier/01-fetch-contract"
sys.path[:0] = [str(ROLE), str(NORMATIVE), str(SUITES), str(FETCH), str(HERE)]
import full_suite_roots as suites  # noqa: E402
import normative_roots as normative  # noqa: E402
import role_aware_source_contract as contract  # noqa: E402
import role_aware_source_evidence as evidence  # noqa: E402
import role_aware_source_lock as source_lock  # noqa: E402
from source_cache import fetch_to_cache  # noqa: E402
from source_model import ContractError, ExternalCache, repository_root  # noqa: E402
from source_role_artifacts import verify_catalog  # noqa: E402
from source_role_records import RoleError  # noqa: E402


class SelectorCacheError(ValueError):
    """The fresh selector cache is partial, stale, mixed, or not atomically published."""


def reject(message: str) -> None:
    raise SelectorCacheError(message)


def records(value: dict[str, object]) -> dict[str, dict[str, object]]:
    items = value["records"]
    assert isinstance(items, list)
    return {str(item["id"]): item for item in items if isinstance(item, dict)}


def source_catalog(value: dict[str, object]) -> dict[str, object]:
    selected = records(value)
    order = ("opengl-46-core-spec", "opengl-cts-gl46-main", "gles-32-spec", "gles-cts-main",
             "vulkan-14-spec", "vulkan-cts-default", "vulkan-registry")
    return {"schema": 1, "records": [selected[identifier] for identifier in order]}


def selector_inputs(value: dict[str, object]):
    selected = records(value)
    return [*(normative.source_input(selected[item]) for item in
              ("opengl-46-core-spec", "gles-32-spec", "vulkan-14-spec", "vulkan-registry")),
            *(suites.source_input(selected[item]) for item in
              ("opengl-cts-gl46-main", "gles-cts-main", "vulkan-cts-default"))]


def fresh_cache(cache_root: Path) -> ExternalCache:
    repository = repository_root(HERE)
    if not isinstance(cache_root, Path) or not cache_root.is_absolute() or cache_root.is_symlink():
        reject("selector cache root must be an absolute nonsymlink path")
    try:
        cache = ExternalCache.from_path(cache_root, repository)
    except ContractError as error:
        reject(str(error))
    if cache.root.exists() and (not cache.root.is_dir() or cache.root.is_symlink() or any(cache.root.iterdir())):
        reject("selector cache root must be missing or empty")
    if cache.root.parent.is_symlink() or not cache.root.parent.is_dir():
        reject("selector cache parent must be an existing regular directory")
    return cache


def expected_files(value: dict[str, object]) -> set[str]:
    inputs = [*selector_inputs(value), *(suites.license_input(item) for item in suites.RELEASE_PROOFS)]
    return {str(item.local_cache) for item in inputs}


def verify_selector_cache(value: dict[str, object], cache_root: Path) -> dict[str, object]:
    try:
        contract.validate_contract(value)
        cache = ExternalCache.from_path(cache_root, repository_root(HERE))
        verify_catalog(source_catalog(value), cache.root)
        selected = records(value)
        normative.verify_notices({item: cache.target(normative.source_input(selected[item])) for item in normative.EXPECTED})
        for proof in suites.RELEASE_PROOFS:
            source = suites.license_input(proof)
            path = cache.target(source)
            if path.is_symlink() or not path.is_file():
                reject("release license is not a regular cache file")
            suites.verify_license_payload(proof, path.read_bytes())
    except (ContractError, RoleError, evidence.EvidenceError, normative.NoticeError,
            normative.RootError, suites.FullSuiteError, contract.SourceContractError) as error:
        reject(str(error))
    found = set()
    for path in cache.root.rglob("*"):
        if path.is_symlink() or not (path.is_dir() or path.is_file()):
            reject("selector cache contains a nonregular path")
        if path.is_file():
            found.add(str(path.relative_to(cache.root)))
    if found != expected_files(value):
        reject("selector cache has missing, stale, or unexpected source files")
    return {"source_contract_sha256": value["source_contract_sha256"],
            "selector_record_ids": [item["id"] for item in source_catalog(value)["records"]],
            "release_proof_ids": [item["license_id"] for item in suites.RELEASE_PROOFS]}


def refresh_root_selectors(value: dict[str, object], cache_root: Path, timeout: float, opener=None) -> dict[str, object]:
    if timeout <= 0:
        reject("timeout must be positive")
    try:
        contract.validate_contract(value)
    except (contract.SourceContractError, evidence.EvidenceError) as error:
        reject(str(error))
    final = fresh_cache(cache_root)
    with tempfile.TemporaryDirectory(prefix=f".{final.root.name}.", dir=final.root.parent) as temporary:
        staged = ExternalCache.from_path(Path(temporary), repository_root(HERE))
        for source in [*selector_inputs(value), *(suites.license_input(item) for item in suites.RELEASE_PROOFS)]:
            try:
                path, reused = fetch_to_cache(staged, source, timeout, opener)
            except ContractError as error:
                reject(str(error))
            if reused or path.is_symlink() or not path.is_file():
                reject("fresh selector refresh reused or failed to create an exact source")
        verify_selector_cache(value, staged.root)
        os.replace(staged.root, final.root)
    return verify_selector_cache(value, final.root)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--selector-cache-root", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=30.0)
    args = parser.parse_args()
    try:
        print(refresh_root_selectors(source_lock.load_locked_contract(), args.selector_cache_root, args.timeout))
    except (SelectorCacheError, source_lock.SourceLockError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
