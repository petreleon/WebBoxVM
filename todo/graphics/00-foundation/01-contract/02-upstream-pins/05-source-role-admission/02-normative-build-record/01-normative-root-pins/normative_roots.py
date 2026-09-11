#!/usr/bin/env python3
"""Immutable F02.5.2.1 normative roots and their external-cache refresher."""

from __future__ import annotations

import argparse
import copy
import sys
from pathlib import Path
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
ROLE = HERE.parents[1] / "01-authority-and-transform-boundary"
FETCH = HERE.parents[2] / "02-fetch-verifier" / "01-fetch-contract"
sys.path[:0] = [str(ROLE), str(FETCH)]
from source_cache import fetch_to_cache  # noqa: E402
from source_model import ContractError, ExternalCache, SourceInput, repository_root  # noqa: E402
from source_role_artifacts import verify_catalog  # noqa: E402
from source_role_contract import validate_catalog  # noqa: E402
from source_role_records import RoleError  # noqa: E402
from normative_notices import NoticeError, verify_notices  # noqa: E402

NO_CLAIMS = {"khronos_selector": False, "api_support": False, "conformance": False,
             "certification": False, "profile_support": False, "performance": False}


class RootError(ValueError):
    """A normative root lacks the exact F02.5.2.1 identity."""


def artifact(identifier: str, digest: str) -> str:
    return f"webboxvm-graphics/f02/{identifier}/{digest}.source"


def root(identifier: str, url: str, revision: str, digest: str, size: int, license: str,
         attribution: str, scope: str) -> dict[str, object]:
    return {"kind": "upstream-source", "id": identifier, "sha256": digest, "bytes": size,
            "license": license, "attribution": attribution, "scope": scope,
            "authority": "Khronos", "producer": "Khronos", "claims": dict(NO_CLAIMS),
            "immutable_url": url, "revision": revision, "artifact": artifact(identifier, digest)}


OPENGL_REVISION = "1cdd228e34966dd6b95bd203e9f84faba0f371a1"
VULKAN_REVISION = "f84d432d5b8912362f96f581f29bbc4f3c8c7843"
ROOTS = (
    root("opengl-46-core-spec", f"https://raw.githubusercontent.com/KhronosGroup/OpenGL-Registry/{OPENGL_REVISION}/specs/gl/glspec46.core.pdf",
         OPENGL_REVISION, "a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee", 3003752,
         "Khronos OpenGL 4.6 Core Profile Specification reproduction terms (PDF file page 3)",
         "Copyright 2006-2022 The Khronos Group Inc.; conditional reproduction notice, PDF file page 3", "normative-source"),
    root("gles-32-spec", f"https://raw.githubusercontent.com/KhronosGroup/OpenGL-Registry/{OPENGL_REVISION}/specs/es/3.2/es_spec_3.2.pdf",
         OPENGL_REVISION, "5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c", 2198754,
         "Khronos OpenGL ES 3.2 Specification reproduction terms (PDF file page 2)",
         "Copyright 2006-2022 The Khronos Group Inc.; conditional reproduction notice, PDF file page 2", "normative-source"),
    root("vulkan-14-spec", f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{VULKAN_REVISION}/vkspec.adoc",
         VULKAN_REVISION, "069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0", 8685,
         "CC-BY-4.0 (vkspec.adoc SPDX-License-Identifier)",
         "Copyright 2014-2026 The Khronos Group Inc.; Vulkan-Docs vkspec.adoc; CC-BY-4.0", "normative-source"),
    root("vulkan-registry", f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{VULKAN_REVISION}/xml/vk.xml",
         VULKAN_REVISION, "cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06", 3309653,
         "Apache-2.0 OR MIT (vk.xml SPDX license choice)",
         "Copyright 2015-2026 The Khronos Group Inc.; Vulkan-Docs xml/vk.xml; Apache-2.0 OR MIT", "registry-metadata"),
)
EXPECTED = {item["id"]: item for item in ROOTS}


def catalog() -> dict[str, object]:
    return {"schema": 1, "records": copy.deepcopy(list(ROOTS))}


def validate_normative_catalog(value: object) -> tuple[str, ...]:
    try:
        identifiers = validate_catalog(value)
    except RoleError as error:
        raise RootError(str(error)) from error
    records = value.get("records") if isinstance(value, dict) else None
    found = {item.get("id"): item for item in records} if isinstance(records, list) else {}
    if set(found) != set(EXPECTED) or len(found) != len(ROOTS):
        raise RootError("normative catalog has incomplete or duplicate root coverage")
    if any(found[identifier] != expected for identifier, expected in EXPECTED.items()):
        raise RootError("normative root differs from its reviewed immutable record")
    return identifiers


def provenance(record: dict[str, object]) -> str:
    parts = urlsplit(record["immutable_url"]).path.split("/")
    return f"https://github.com/KhronosGroup/{parts[2]}/tree/{record['revision']}"


def source_input(record: dict[str, object]) -> SourceInput:
    return SourceInput.from_manifest({"id": record["id"], "source_family": "f0252-normative-root",
                                      "immutable_url": record["immutable_url"], "revision": record["revision"],
                                      "sha256": record["sha256"], "bytes": record["bytes"],
                                      "license": record["license"], "local_cache": record["artifact"],
                                      "generated_code_role": "Immutable F02.5.2.1 source root; no generated code",
                                      "provenance": provenance(record)})


def require_fresh_cache(root: Path) -> None:
    if root.exists() and (not root.is_dir() or any(root.iterdir())):
        raise RootError("fresh cache root must be missing or an empty directory")


def fetch_and_verify(value: object, cache_root: Path, timeout: float) -> list[tuple[str, str, Path]]:
    if timeout <= 0:
        raise RootError("timeout must be positive")
    validate_normative_catalog(value)
    cache = ExternalCache.from_path(cache_root, repository_root(HERE))
    require_fresh_cache(cache.root)
    results = []
    for record in value["records"]:
        path, reused = fetch_to_cache(cache, source_input(record), timeout)
        if reused:
            raise RootError("fresh source refresh unexpectedly reused a cache target")
        results.append((record["id"], "reused" if reused else "fetched", path))
    verify_catalog(value, cache.root)
    try:
        verify_notices({identifier: path for identifier, _state, path in results})
    except NoticeError as error:
        raise RootError(str(error)) from error
    return results


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=30.0)
    args = parser.parse_args()
    try:
        for identifier, state, path in fetch_and_verify(catalog(), args.cache_root, args.timeout):
            print(f"PASS: {identifier} {state} {path}")
    except (ContractError, NoticeError, RoleError, RootError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
