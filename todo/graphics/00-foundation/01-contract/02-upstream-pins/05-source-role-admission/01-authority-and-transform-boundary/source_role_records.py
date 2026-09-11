#!/usr/bin/env python3
"""Role-specific F02.5 record validation, separate from catalog graph checks."""

from __future__ import annotations

import re
import sys
from pathlib import Path, PurePosixPath
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "02-fetch-verifier" / "01-fetch-contract"))
from source_model import ContractError, immutable_url  # noqa: E402

MAX_LOCAL_BYTES = 8 * 1024 * 1024
ID = re.compile(r"^[a-z0-9][a-z0-9-]*$")
DIGEST = re.compile(r"^[0-9a-f]{64}$")
COMMIT = re.compile(r"^[0-9a-f]{40}$")
CLAIMS = frozenset(("khronos_selector", "api_support", "conformance", "certification",
                    "profile_support", "performance"))
COMMON = frozenset(("kind", "id", "sha256", "bytes", "license", "attribution", "scope",
                    "authority", "producer", "claims"))
UPSTREAM = COMMON | frozenset(("immutable_url", "revision", "artifact"))
MEMBER = UPSTREAM | frozenset(("suite_root_id", "member_path"))
TRANSFORM = COMMON | frozenset(("inputs", "command", "artifact", "builder"))
SHARD = TRANSFORM | frozenset(("shard",))
FULL_SUITE = UPSTREAM | frozenset(("profile", "suite_id", "selector_path", "unfiltered"))
UPSTREAM_SCOPES = frozenset(("normative-source", "registry-metadata", "suite-member"))
LOCAL_SCOPES = frozenset(("webboxvm-core-definition", "webboxvm-derived-docs-attestation",
                          "webboxvm-engineering-map", "webboxvm-byte-preserving-shard"))
FULL_SUITES = {
    "opengl-4.6-core": ("opengl-cts-gl46-main", "VK-GL-CTS",
                          "external/openglcts/data/gl_cts/data/mustpass/gl/khronos_mustpass/4.6.1.x/gl46-main.txt"),
    "gles-3.2": ("gles-cts-main", "VK-GL-CTS",
                  "external/openglcts/data/gl_cts/data/mustpass/gles/khronos_mustpass/main/mustpass.xml"),
    "vulkan-1.4-core": ("vulkan-cts-default", "VK-GL-CTS",
                         "external/vulkancts/mustpass/main/vk-default.txt"),
}


class RoleError(ValueError):
    """A record would conflate source authority with qualification."""

def reject(message: str) -> None:
    raise RoleError(message)

def text(value: object, name: str) -> str:
    if not isinstance(value, str) or not value:
        reject(f"{name} must be a nonempty string")
    return value

def identity(value: object, name: str = "id") -> str:
    value = text(value, name)
    if not ID.fullmatch(value):
        reject(f"{name} has an invalid shape")
    return value

def digest(value: object, name: str = "sha256") -> str:
    value = text(value, name)
    if not DIGEST.fullmatch(value) or value == "0" * 64:
        reject(f"{name} must be a nonzero SHA-256")
    return value


def positive(value: object, name: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        reject(f"{name} must be a positive integer")
    return value


def artifact(value: object, name: str = "artifact") -> str:
    value = text(value, name)
    path = PurePosixPath(value)
    if path.is_absolute() or str(path) != value or not path.parts or any(part in (".", "..") for part in path.parts):
        reject(f"{name} must be a safe relative artifact key")
    return value


def fields(value: object, expected: frozenset[str], name: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != expected:
        reject(f"{name} has an invalid schema")
    return value


def claim_set(value: object, selector: bool) -> None:
    value = fields(value, CLAIMS, "claims")
    if any(type(item) is not bool for item in value.values()):
        reject("claims must contain only booleans")
    if value["khronos_selector"] is not selector:
        reject("khronos_selector does not match the source role")
    if any(value[name] for name in CLAIMS - {"khronos_selector"}):
        reject("source roles cannot claim support, execution, certification, or performance")


def common(record: dict[str, object], selector: bool) -> str:
    identifier = identity(record["id"])
    digest(record["sha256"])
    positive(record["bytes"], "bytes")
    artifact(record["artifact"])
    for name in ("license", "attribution", "scope", "authority", "producer"):
        text(record[name], name)
    claim_set(record["claims"], selector)
    return identifier


def upstream_identity(record: dict[str, object]) -> tuple[str, str, str]:
    identifier = identity(record["id"])
    revision = text(record["revision"], "revision")
    if not COMMIT.fullmatch(revision):
        reject("revision must be a 40-hex commit")
    url = text(record["immutable_url"], "immutable_url")
    try:
        immutable_url(url, revision, identifier)
    except ContractError as error:
        reject(str(error))
    parsed = urlsplit(url)
    parts = parsed.path.lstrip("/").split("/")
    if parsed.hostname != "raw.githubusercontent.com" or len(parts) < 4 or parts[0] != "KhronosGroup":
        reject("immutable_url must name a KhronosGroup raw GitHub source")
    return revision, parts[1], "/".join(parts[3:])


def builder(value: object) -> None:
    value = fields(value, frozenset(("id", "sha256", "artifact")), "builder")
    if value["id"] != "webboxvm-source-builder":
        reject("builder must use the WebBoxVM source-builder contract")
    digest(value["sha256"], "builder sha256")
    artifact(value["artifact"], "builder artifact")


def argv(value: object) -> None:
    if not isinstance(value, list) or not value:
        reject("command must be a nonempty argv list")
    for part in value:
        if not isinstance(part, str) or not part or part != part.strip() or "\0" in part or "://" in part:
            reject("command argv has an invalid token")
    if value[0] != "webboxvm-source-builder":
        reject("command must use the WebBoxVM source-builder contract")


def record_shape(record: object) -> str:
    if not isinstance(record, dict):
        reject("record must be an object")
    kind = record.get("kind")
    expected = {"upstream-source": UPSTREAM, "webboxvm-transform": TRANSFORM,
                "full-suite-root": FULL_SUITE}.get(kind)
    if kind == "upstream-source" and record.get("scope") == "suite-member":
        expected = MEMBER
    if kind == "webboxvm-transform" and record.get("scope") == "webboxvm-byte-preserving-shard":
        expected = SHARD
    if expected is None:
        reject("record has an unknown role")
    fields(record, expected, "record")
    identifier = common(record, kind == "full-suite-root")
    if kind == "upstream-source":
        _revision, _repository, path = upstream_identity(record)
        if (record["authority"], record["producer"]) != ("Khronos", "Khronos") or record["scope"] not in UPSTREAM_SCOPES:
            reject("upstream source has an invalid authority boundary")
        if record["scope"] == "suite-member":
            identity(record["suite_root_id"], "suite_root_id")
            if text(record["member_path"], "member_path") != path:
                reject("suite member has an invalid root anchor or path")
    elif kind == "webboxvm-transform":
        if (record["authority"], record["producer"]) != ("WebBoxVM", "WebBoxVM") or record["scope"] not in LOCAL_SCOPES:
            reject("transform has an invalid authority boundary")
        if record["bytes"] > MAX_LOCAL_BYTES:
            reject("WebBoxVM transform exceeds 8 MiB")
        builder(record["builder"])
        argv(record["command"])
    else:
        _revision, repository, path = upstream_identity(record)
        expected_root = FULL_SUITES.get(text(record["profile"], "profile"))
        if type(record["unfiltered"]) is not bool or record["unfiltered"] is not True or expected_root is None:
            reject("full suite root has an invalid profile or filtering boundary")
        suite_id, expected_repo, expected_path = expected_root
        if (record["authority"], record["producer"], record["scope"], record["suite_id"], repository, path,
                record["selector_path"]) != ("Khronos", "Khronos", "full-conformance-suite", suite_id,
                                               expected_repo, expected_path, expected_path):
            reject("full suite root is not its profile's canonical upstream selector")
    return identifier
