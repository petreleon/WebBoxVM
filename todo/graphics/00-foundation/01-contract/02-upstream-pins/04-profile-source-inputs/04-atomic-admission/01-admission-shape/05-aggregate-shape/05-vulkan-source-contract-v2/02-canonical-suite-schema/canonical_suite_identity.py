#!/usr/bin/env python3
"""Freeze the V2 identity of the Khronos Vulkan CTS default suite root."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent
SOURCE_API_PATH = HERE.parents[4] / "audit_source_api.py"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
COMPONENT = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")
PREFIX = ("external", "vulkancts", "mustpass", "main")
IDENTITY_FIELDS = frozenset(("schema", "kind", "suite_id", "repository_url", "tag_name", "tag_ref",
                             "tag_object_sha1", "peeled_commit_sha1", "root_path", "root_input",
                             "direct_member_count", "direct_member_paths", "direct_member_paths_sha256",
                             "selector_scope", "local_filtering", "identity_sha256"))
EXPECTED = {
    "schema": 1, "kind": "canonical-upstream-suite-root", "suite_id": "vulkan-cts-mustpass",
    "repository_url": "https://github.com/KhronosGroup/VK-GL-CTS", "tag_name": "vulkan-cts-1.4.6.2",
    "tag_ref": "refs/tags/vulkan-cts-1.4.6.2", "tag_object_sha1": "42c723aa10d2652590f02741827aef43b0421d23",
    "peeled_commit_sha1": "f6a29701220f34dd1407513bfe80d74ca7b392ce",
    "root_path": "external/vulkancts/mustpass/main/vk-default.txt", "direct_member_count": 98,
    "selector_scope": "khronos-default-mustpass-broader-than-vulkan-1.4-core", "local_filtering": "forbidden",
}
EXPECTED_PATHS_SHA256 = "9f0e41be554a324bb0db2137a2fb44775689c6e5bf4bc70c1b4f8988d37f7b9a"
EXPECTED_IDENTITY_SHA256 = "30b272f8c563e0dabf307795c01496eb70f744514b4790439ebbfc69c7bd5218"
ROOT = {
    "id": "vulkan-cts-mustpass", "source_family": "vulkan-cts",
    "immutable_url": "https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/f6a29701220f34dd1407513bfe80d74ca7b392ce/external/vulkancts/mustpass/main/vk-default.txt",
    "revision": "f6a29701220f34dd1407513bfe80d74ca7b392ce",
    "sha256": "b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4",
    "bytes": 3347, "license": "Apache-2.0 (VK-GL-CTS repository LICENSE)",
    "local_cache": "webboxvm-graphics/f02/vulkan-cts-mustpass/b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4.source",
    "generated_code_role": "Vulkan CTS must-pass root selector; no code is imported",
    "provenance": "https://github.com/KhronosGroup/VK-GL-CTS/tree/f6a29701220f34dd1407513bfe80d74ca7b392ce",
}


class IdentityError(ValueError):
    """The V2 root identity is malformed, mutable, or substituted."""


def reject(message: str) -> None:
    raise IdentityError(message)


def object_without_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in pairs:
        if key in value:
            reject("identity has a duplicate JSON key")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=object_without_duplicates)
    except IdentityError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"identity cannot be read: {error}")
    if not isinstance(value, dict):
        reject("identity is not an object")
    return value


def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "identity_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode("utf-8")).hexdigest()


def paths_digest(paths: tuple[str, ...]) -> str:
    return hashlib.sha256(json.dumps(paths, separators=(",", ":")).encode("utf-8")).hexdigest()


def safe_path(value: object) -> str:
    if not isinstance(value, str) or not value:
        reject("identity direct member path is empty")
    path = PurePosixPath(value)
    if (path.is_absolute() or str(path) != value or path.parts[:4] != PREFIX or path.suffix != ".txt"
            or any(not COMPONENT.fullmatch(part) for part in path.parts)):
        reject("identity direct member path is unsafe or outside the VCTS mustpass root")
    return value


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load reviewed F02 source API")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


SOURCE_API = module("f025_v2_source_api", SOURCE_API_PATH)
_, _, ContractError, SourceInput = SOURCE_API.source_api()


@dataclass(frozen=True)
class SuiteIdentity:
    suite_id: str
    digest: str
    peeled_commit: str
    root_path: str
    direct_member_count: int
    direct_members: tuple[str, ...]


def exact(value: dict[str, object], fields: set[str] | frozenset[str], label: str) -> None:
    if set(value) != fields:
        reject(f"identity {label} has an unexpected schema")


def validate(path: Path) -> SuiteIdentity:
    value = document(path)
    exact(value, IDENTITY_FIELDS, "root")
    if any(value[key] != expected for key, expected in EXPECTED.items()):
        reject("identity does not match the reviewed VCTS root")
    if type(value["schema"]) is not int or type(value["direct_member_count"]) is not int:
        reject("identity has an invalid numeric field")
    tag_object, commit = value["tag_object_sha1"], value["peeled_commit_sha1"]
    if not isinstance(tag_object, str) or not isinstance(commit, str) or not HEX40.fullmatch(tag_object) or not HEX40.fullmatch(commit):
        reject("identity has an invalid tag or commit hash")
    if tag_object == commit:
        reject("identity annotated tag object must differ from its peeled commit")
    root = value["root_input"]
    if not isinstance(root, dict) or root != ROOT:
        reject("identity root input does not match the reviewed VCTS descriptor")
    try:
        parsed = SourceInput.from_manifest(root)
    except ContractError as error:
        reject(f"identity root violates F02.2 regular-source policy: {error}")
    if parsed.revision != commit or not parsed.url.endswith("/" + str(value["root_path"])):
        reject("identity root does not bind the peeled commit and selector path")
    count = value["direct_member_count"]
    if type(count) is not int or count < 1:
        reject("identity has an invalid direct member count")
    paths = value["direct_member_paths"]
    if not isinstance(paths, list) or len(paths) != count:
        reject("identity has an incomplete direct member list")
    direct_members = tuple(safe_path(item) for item in paths)
    if len(set(direct_members)) != count:
        reject("identity repeats a direct member path")
    paths_hash = value["direct_member_paths_sha256"]
    if (not isinstance(paths_hash, str) or not HEX64.fullmatch(paths_hash)
            or paths_hash != EXPECTED_PATHS_SHA256 or paths_hash != paths_digest(direct_members)):
        reject("identity has a stale direct member list")
    claimed = value["identity_sha256"]
    if (not isinstance(claimed, str) or not HEX64.fullmatch(claimed) or claimed != EXPECTED_IDENTITY_SHA256
            or claimed != digest(value)):
        reject("identity has a stale canonical digest")
    return SuiteIdentity(str(value["suite_id"]), claimed, commit, str(value["root_path"]), count, direct_members)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: canonical_suite_identity.py IDENTITY.json")
    try:
        identity = validate(Path(sys.argv[1]))
    except IdentityError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"IDENTITY: {identity.suite_id} {identity.peeled_commit} {identity.digest}")


if __name__ == "__main__":
    main()
