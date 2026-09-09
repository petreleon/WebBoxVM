#!/usr/bin/env python3
"""Fail-closed validation of the Vulkan CTS root-selector reference list."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent
PARENT = HERE.parent
ROOT_FIELDS = frozenset(("schema", "profile", "candidate_id", "candidate_sha256", "direct_references",
                         "direct_reference_count", "scope_boundary", "scope_boundary_examples"))
SCOPE_BOUNDARY = "broader-upstream-default-includes-wsi-and-extension-groups"
SCOPE_EXAMPLES = ("vk-default/wsi.txt", "vk-default/video.txt", "vk-default/ray-query.txt",
                  "vk-default/cooperative-vector.txt", "vk-default/data-graph.txt", "vk-default/tensor.txt")


class ReferenceError(ValueError):
    """A Vulkan CTS direct-selector list is unsafe or stale."""


def reject(message: str) -> None:
    raise ReferenceError(message)


def reviewed_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed module: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(spec.name, None)
        raise
    return module


CANDIDATES = reviewed_module("f024_candidate_contract_for_vulkan_refs", PARENT / "candidate_contract.py")


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"must-pass references cannot be read: {error}")
    if not isinstance(value, dict):
        reject("must-pass references are not a JSON object")
    return value


def expected_candidate(profile: str, candidates: Path) -> dict[str, object]:
    identity = CANDIDATES.CATALOG.get(profile, {}).get("vulkan-cts-mustpass")
    if not isinstance(identity, dict):
        reject("must-pass references have an unknown target profile")
    try:
        requirements = CANDIDATES.requirements(profile)
        decisions = CANDIDATES.validate(candidates, profile)
    except CANDIDATES.AuditError as error:
        reject(f"must-pass candidate contract failed: {error}")
    matching = [decision for (identifier, role), decision in zip(requirements, decisions)
                if identifier == "vulkan-cts-mustpass" and role == "conformance-manifest"]
    if matching != ["rejected"] or identity.get("decision") != "rejected":
        reject("must-pass references do not bind a rejected candidate")
    return identity


def references(value: object, digest: object, expected_count: object) -> tuple[str, ...]:
    if not isinstance(value, list) or type(expected_count) is not int or expected_count < 1:
        reject("must-pass references have an invalid list or count")
    if len(value) != expected_count or any(not isinstance(item, str) for item in value):
        reject("must-pass references have an incorrect count")
    result = tuple(value)
    for item in result:
        path = PurePosixPath(item)
        if path.is_absolute() or str(path) != item or ".." in path.parts or path.parts[:1] != ("vk-default",) or path.suffix != ".txt":
            reject("must-pass reference path is unsafe")
    if len(set(result)) != len(result):
        reject("must-pass references repeat a path")
    rendered = ("\n".join(result) + "\n").encode("utf-8")
    if hashlib.sha256(rendered).hexdigest() != digest:
        reject("must-pass references do not reproduce the reviewed root bytes")
    return result


def validate(path: Path, candidates: Path, profile: str) -> int:
    identity = expected_candidate(profile, candidates)
    value = document(path)
    if set(value) != ROOT_FIELDS or type(value.get("schema")) is not int or value["schema"] != 1 or value.get("profile") != profile:
        reject("must-pass references do not match schema version 1")
    if value.get("candidate_id") != "vulkan-cts-mustpass" or value.get("candidate_sha256") != identity.get("sha256"):
        reject("must-pass references do not bind the reviewed root")
    count = value.get("direct_reference_count")
    if type(count) is not int or count != identity.get("selector_case_count"):
        reject("must-pass references have a stale direct_reference_count")
    result = references(value.get("direct_references"), identity["sha256"], count)
    examples = value.get("scope_boundary_examples")
    if value.get("scope_boundary") != SCOPE_BOUNDARY or not isinstance(examples, list) or tuple(examples) != SCOPE_EXAMPLES:
        reject("must-pass references do not bind the core scope boundary")
    if not set(examples).issubset(result):
        reject("must-pass scope examples are absent from the reviewed root")
    return len(result)


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit("usage: mustpass_reference_contract.py REFERENCES.json CANDIDATES.json PROFILE")
    try:
        count = validate(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3])
    except ReferenceError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"REFERENCES: {sys.argv[3]} {count} direct must-pass selectors")


if __name__ == "__main__":
    main()
