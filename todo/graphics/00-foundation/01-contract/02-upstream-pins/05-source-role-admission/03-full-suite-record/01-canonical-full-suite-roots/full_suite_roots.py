#!/usr/bin/env python3
"""Pin and refresh the three Khronos full-suite roots without local filtering."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROLE = HERE.parents[1] / "01-authority-and-transform-boundary"
FETCH = HERE.parents[2] / "02-fetch-verifier" / "01-fetch-contract"
sys.path[:0] = [str(ROLE), str(FETCH)]
from source_cache import fetch_to_cache  # noqa: E402
from source_model import ContractError, ExternalCache, SourceInput, repository_root  # noqa: E402
from source_role_artifacts import verify_catalog  # noqa: E402
from source_role_contract import validate_catalog  # noqa: E402
from source_role_records import RoleError  # noqa: E402

LICENSE = "Apache-2.0 (VK-GL-CTS repository LICENSE)"
ATTRIBUTION = "KhronosGroup/VK-GL-CTS repository; immutable upstream full-suite selector"
CLAIMS = {"khronos_selector": True, "api_support": False, "conformance": False,
          "certification": False, "profile_support": False, "performance": False}
NO_CLAIMS = {name: False for name in CLAIMS}
GL_REVISION = "067e8832315e79817ede1c4863804e440f5d1c80"
VK_REVISION = "f6a29701220f34dd1407513bfe80d74ca7b392ce"
LICENSE_SHA256 = "cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30"
LICENSE_BYTES = 11358

class FullSuiteError(ValueError):
    """A full suite root would lose its exact Khronos identity."""


def reject(message: str) -> None:
    raise FullSuiteError(message)


def cache_key(identifier: str, digest: str) -> str:
    return f"webboxvm-graphics/f02/{identifier}/{digest}.source"


def root(identifier: str, profile: str, revision: str, digest: str, size: int, path: str) -> dict[str, object]:
    return {"kind": "full-suite-root", "id": identifier, "sha256": digest, "bytes": size,
            "license": LICENSE, "attribution": ATTRIBUTION, "scope": "full-conformance-suite",
            "authority": "Khronos", "producer": "Khronos", "claims": dict(CLAIMS),
            "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/{revision}/{path}",
            "revision": revision, "artifact": cache_key(identifier, digest), "profile": profile,
            "suite_id": identifier, "selector_path": path, "unfiltered": True}


ROOTS = (
    root("opengl-cts-gl46-main", "opengl-4.6-core", GL_REVISION,
         "e28bbbbfd0f6c8d711554a01aa45819bdc7ca963c9426e997dfd1daaeb1d7b17", 1353085,
         "external/openglcts/data/gl_cts/data/mustpass/gl/khronos_mustpass/4.6.1.x/gl46-main.txt"),
    root("gles-cts-main", "gles-3.2", GL_REVISION,
         "9f466f19a26120149bab06e70596c9fff57b61d767cfdbadc7ba13934ce2ea96", 4284,
         "external/openglcts/data/gl_cts/data/mustpass/gles/khronos_mustpass/main/mustpass.xml"),
    root("vulkan-cts-default", "vulkan-1.4-core", VK_REVISION,
         "b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4", 3347,
         "external/vulkancts/mustpass/main/vk-default.txt"),
)
ROOT_IDS = tuple(item["id"] for item in ROOTS)
RELEASE_PROOFS = (
    {"tag": "opengl-cts-4.6.8.1", "tag_object": "f7eefdfcae4a19fa69ad7df0c00da8c1a65723a3",
     "peeled_commit": GL_REVISION, "profiles": ("opengl-4.6-core", "gles-3.2"),
     "license_id": "opengl-cts-4681-license"},
    {"tag": "vulkan-cts-1.4.6.2", "tag_object": "42c723aa10d2652590f02741827aef43b0421d23",
     "peeled_commit": VK_REVISION, "profiles": ("vulkan-1.4-core",),
     "license_id": "vulkan-cts-1462-license"},
)


def catalog() -> dict[str, object]:
    return {"schema": 1, "records": copy.deepcopy(list(ROOTS))}


def proof_for(record: dict[str, object]) -> dict[str, object]:
    matches = [proof for proof in RELEASE_PROOFS if record["profile"] in proof["profiles"]]
    if len(matches) != 1 or matches[0]["peeled_commit"] != record["revision"]:
        reject("full-suite root does not bind to exactly one reviewed release proof")
    return matches[0]


def validate_full_suite_catalog(value: object) -> tuple[str, ...]:
    try:
        identifiers = validate_catalog(value)
    except RoleError as error:
        reject(str(error))
    if identifiers != ROOT_IDS or value != catalog():
        reject("full-suite catalog differs from the exact ordered root record")
    for record in value["records"]:
        proof_for(record)
    return identifiers


def source_input(record: dict[str, object]) -> SourceInput:
    return SourceInput.from_manifest({
        "id": record["id"], "source_family": "f0253-full-suite-root",
        "immutable_url": record["immutable_url"], "revision": record["revision"],
        "sha256": record["sha256"], "bytes": record["bytes"], "license": record["license"],
        "local_cache": record["artifact"], "generated_code_role": "Immutable full-suite selector; no generated code",
        "provenance": f"https://github.com/KhronosGroup/VK-GL-CTS/tree/{record['revision']}",
    })


def license_input(proof: dict[str, object]) -> SourceInput:
    identifier, revision = proof["license_id"], proof["peeled_commit"]
    return SourceInput.from_manifest({
        "id": identifier, "source_family": "f0253-release-license",
        "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/{revision}/LICENSE",
        "revision": revision, "sha256": LICENSE_SHA256, "bytes": LICENSE_BYTES, "license": LICENSE,
        "local_cache": cache_key(identifier, LICENSE_SHA256), "generated_code_role": "Release license proof; no generated code",
        "provenance": f"https://github.com/KhronosGroup/VK-GL-CTS/tree/{revision}",
    })


def verify_license_payload(proof: dict[str, object], payload: bytes) -> None:
    if (len(payload), hashlib.sha256(payload).hexdigest()) != (LICENSE_BYTES, LICENSE_SHA256):
        reject(f"{proof['tag']} LICENSE has an unexpected identity")
    if b"Apache License" not in payload or b"Version 2.0" not in payload:
        reject(f"{proof['tag']} LICENSE lacks the Apache 2.0 notice")


def require_fresh_cache(root: Path) -> None:
    if root.exists() and (not root.is_dir() or root.is_symlink() or any(root.iterdir())):
        reject("fresh cache root must be missing or an empty regular directory")


def fetch_and_verify(value: object, cache_root: Path, timeout: float) -> dict[str, Path]:
    if timeout <= 0:
        reject("timeout must be positive")
    validate_full_suite_catalog(value)
    cache = ExternalCache.from_path(cache_root, repository_root(HERE))
    require_fresh_cache(cache.root)
    paths = {}
    for record in value["records"]:
        path, reused = fetch_to_cache(cache, source_input(record), timeout)
        if reused:
            reject("fresh full-suite refresh unexpectedly reused a root")
        paths[record["id"]] = path
    try:
        verify_catalog(value, cache.root)
    except RoleError as error:
        reject(str(error))
    for proof in RELEASE_PROOFS:
        path, reused = fetch_to_cache(cache, license_input(proof), timeout)
        if reused:
            reject("fresh full-suite refresh unexpectedly reused a release license")
        if path.is_symlink() or not path.is_file():
            reject("release license is not a regular file")
        verify_license_payload(proof, path.read_bytes())
        paths[proof["license_id"]] = path
    return paths


def receipt(value: object) -> dict[str, object]:
    validate_full_suite_catalog(value)
    return {"schema": 1, "authority": "WebBoxVM", "producer": "WebBoxVM", "claims": dict(NO_CLAIMS),
            "cts_executions": 0, "root_ids": list(ROOT_IDS), "release_proofs": copy.deepcopy(list(RELEASE_PROOFS)),
            "vulkan_selector_scope": "Khronos vk-default root; broader than a Vulkan-1.4-core-only selector"}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=30.0)
    args = parser.parse_args()
    try:
        fetch_and_verify(catalog(), args.cache_root, args.timeout)
        print(json.dumps(receipt(catalog()), sort_keys=True))
    except (ContractError, FullSuiteError, RoleError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)

if __name__ == "__main__":
    main()
