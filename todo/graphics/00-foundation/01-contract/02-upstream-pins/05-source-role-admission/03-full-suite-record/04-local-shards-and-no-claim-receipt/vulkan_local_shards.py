#!/usr/bin/env python3
"""Build and verify five local, byte-preserving shards from verified VCTS cache data."""

from __future__ import annotations

import copy
import os
import shutil
import stat
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROLE = HERE.parents[1] / "01-authority-and-transform-boundary"
LEDGER = HERE.parent / "03-vulkan-default-full-ledger/01-vulkan-ledger-taxonomy"
REPLAY = HERE.parent / "03-vulkan-default-full-ledger/02-streaming-cache-replay"
sys.path[:0] = [str(ROLE), str(LEDGER), str(REPLAY)]
from source_role_artifacts import verify_catalog  # noqa: E402
from source_role_records import RoleError  # noqa: E402
import vulkan_cache_replay as replay  # noqa: E402
import vulkan_ledger_taxonomy as ledger_map  # noqa: E402
import webboxvm_byte_preserving_shard_builder as builder  # noqa: E402

API_ID = "vulkan-cts-default-api"
API_PATH = "external/vulkancts/mustpass/main/vk-default/api.txt"
API_BLOB_SHA1 = "9440aee9b3291b2927119c97ab44e2858a63a39b"
API = (builder.API_BYTES, builder.API_SHA256)
ROOT_ARTIFACT, API_ARTIFACT = "sources/vk-default.txt", "sources/api.txt"
BUILDER_ARTIFACT = "tools/webboxvm-byte-preserving-shard-builder.py"
SHARDS = ((0, 8388608, "0cefa07a54d2b2fac2f0238af6dd1831be1ed9055c6fa251e5d1f8c71f98eb6a"),
          (8388608, 8388608, "b19278faa0324968199d25a45a960988457a3537d38956b5d28a2fd96645eba5"),
          (16777216, 8388608, "a213735b7d3b25f3509e0fa6df88fd8060edeba4361a6877dd9809c2d5e1b625"),
          (25165824, 8388608, "1403a2c14aee1f8cf12ac6f1ef95b48eefe8b31d665964f81b2bb8088f40e38e"),
          (33554432, 6741627, "fa6ba8875dd181410b4a5da6ac4b192e7e5ab14ff2ad4ba136b40c84f39b38c5"))
NO_CLAIMS = {"khronos_selector": False, "api_support": False, "conformance": False,
             "certification": False, "profile_support": False, "performance": False}


class ShardError(ValueError):
    """VCTS provenance or local shard bytes are not exact."""


def reject(message: str) -> None:
    raise ShardError(message)


def bridge(cache_root: Path) -> tuple[dict[str, object], dict[str, object], dict[str, object]]:
    try:
        checked, observed = replay.replay(cache_root), ledger_map.build()
    except (ledger_map.VulkanLedgerError, replay.ReplayError) as error:
        reject(str(error))
    root, ledger = observed["source_root"], observed["ledger"]
    assert isinstance(root, dict) and isinstance(ledger, dict)
    row = next((item for item in ledger["members"] if item["path"] == API_PATH), None)
    expected = ("f6a29701220f34dd1407513bfe80d74ca7b392ce", API_BLOB_SHA1, *API)
    actual = None if not isinstance(row, dict) else (row.get("revision"), row.get("blob_sha1"), row.get("bytes"), row.get("sha256"))
    if (actual != expected or not replay.alike(checked.get("source_root"), root)
            or (checked.get("ledger_sha256"), checked.get("member_count"), checked.get("member_total_bytes")) !=
            (ledger["ledger_sha256"], ledger["member_count"], ledger["member_total_bytes"])):
        reject("VCTS cache replay does not bind the exact vk-default api.txt member")
    return root, ledger, checked


def cache_file(cache_root: Path, ledger: dict[str, object], name: str) -> Path:
    prefix = cache_root / "webboxvm-graphics/v2/vulkan-cts-mustpass"
    path = prefix / str(ledger["identity_sha256"]) / str(ledger["ledger_sha256"]) / name
    if path.is_symlink() or not path.is_file():
        reject("verified VCTS cache source is not a regular file")
    return path


def fresh_root(value: Path, cache_root: Path) -> Path:
    if not value.is_absolute() or value.is_symlink():
        reject("artifact root must be an absolute non-symlink path")
    value.mkdir(mode=0o700, parents=True, exist_ok=True)
    state = value.stat()
    if (not stat.S_ISDIR(state.st_mode) or state.st_uid != os.geteuid() or state.st_mode & 0o077
            or any(value.iterdir())):
        reject("artifact root must be a fresh private directory")
    root, cache = value.resolve(), cache_root.resolve()
    if root == cache or root in cache.parents or cache in root.parents:
        reject("artifact root must not overlap the VCTS cache")
    return root


def stage(source: Path, destination: Path, expected: tuple[int, str]) -> None:
    if source.is_symlink() or not source.is_file() or destination.exists() or destination.is_symlink():
        reject("staged artifact must have one regular source and absent destination")
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.parent.is_symlink():
        reject("staged artifact parent cannot be a symlink")
    shutil.copyfile(source, destination)
    if builder.digest_file(destination) != expected:
        reject("staged artifact does not retain its pinned bytes")


def member(root: dict[str, object]) -> dict[str, object]:
    revision = str(root["revision"])
    return {"kind": "upstream-source", "id": API_ID, "sha256": API[1], "bytes": API[0],
            "license": root["license"], "attribution": "KhronosGroup/VK-GL-CTS immutable api.txt suite member",
            "scope": "suite-member", "authority": "Khronos", "producer": "Khronos", "claims": dict(NO_CLAIMS),
            "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/{revision}/{API_PATH}",
            "revision": revision, "artifact": API_ARTIFACT, "suite_root_id": root["id"], "member_path": API_PATH}


def catalog(root: dict[str, object]) -> dict[str, object]:
    source = copy.deepcopy(root)
    source["artifact"] = ROOT_ARTIFACT
    builder_path = Path(builder.__file__).resolve()
    builder_pin = {"id": "webboxvm-source-builder", "sha256": builder.digest_file(builder_path)[1],
                   "artifact": BUILDER_ARTIFACT}
    records: list[dict[str, object]] = [source, member(source)]
    for index, (offset, size, digest) in enumerate(SHARDS):
        identifier = f"vulkan-cts-api-shard-{index}"
        records.append({"kind": "webboxvm-transform", "id": identifier, "sha256": digest, "bytes": size,
                        "license": source["license"], "attribution": "Khronos api.txt; WebBoxVM byte-preserving shard",
                        "scope": "webboxvm-byte-preserving-shard", "authority": "WebBoxVM", "producer": "WebBoxVM",
                        "claims": dict(NO_CLAIMS), "inputs": [{"id": API_ID, "kind": "upstream-source", "sha256": API[1],
                                                                     "revision": source["revision"]}],
                        "command": ["webboxvm-source-builder", "--mode=byte-preserving-shard", f"--shard-index={index}",
                                    f"--shard-count={len(SHARDS)}", f"--shard-offset={offset}", f"--shard-bytes={size}",
                                    f"@input:{API_ID}", f"@output:{identifier}"], "artifact": f"shards/api-{index}.source",
                        "builder": dict(builder_pin), "shard": {"source_id": API_ID, "source_sha256": API[1],
                        "source_bytes": API[0], "index": index, "count": len(SHARDS), "offset": offset,
                        "reassembled_sha256": API[1]}})
    return {"schema": 1, "records": records}


def verify(value: object, artifact_root: Path, root: dict[str, object]) -> dict[str, object]:
    if value != catalog(root):
        reject("shard catalog is not the exact ordered VCTS api.txt transform")
    try:
        verify_catalog(value, artifact_root)
    except RoleError as error:
        reject(str(error))
    return value


def build(cache_root: Path, artifact_root: Path) -> tuple[dict[str, object], dict[str, object]]:
    root, ledger, checked = bridge(cache_root)
    destination = fresh_root(artifact_root, cache_root)
    stage(cache_file(cache_root, ledger, "root/" + str(root["sha256"]) + ".source"), destination / ROOT_ARTIFACT,
          (int(root["bytes"]), str(root["sha256"])))
    source = destination / API_ARTIFACT
    stage(cache_file(cache_root, ledger, "members/" + API[1] + ".source"), source, API)
    stage(Path(builder.__file__).resolve(), destination / BUILDER_ARTIFACT, builder.digest_file(Path(builder.__file__).resolve()))
    for index, (offset, size, digest) in enumerate(SHARDS):
        if builder.build(source, destination / f"shards/api-{index}.source", index, len(SHARDS), offset, size) != (size, digest):
            reject("new local shard differs from the pinned byte-preserving identity")
    return verify(catalog(root), destination, root), checked


def validate(cache_root: Path, artifact_root: Path) -> tuple[dict[str, object], dict[str, object]]:
    root, _ledger, checked = bridge(cache_root)
    return verify(catalog(root), artifact_root, root), checked
