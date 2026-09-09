#!/usr/bin/env python3
"""Fail-closed validation of the Vulkan specification root include closure."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent
PARENT = HERE.parent
ROOT_FIELDS = frozenset(("schema", "profile", "candidate_id", "candidate_sha256", "direct_includes",
                         "direct_include_count", "include_list_sha256", "closure_boundary", "boundary_examples"))
CANDIDATE_ID = "vulkan-14-spec"
CANDIDATE_SHA256 = "069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0"
INCLUDE_COUNT = 73
INCLUDE_SHA256 = "b35c3b8907a3338e688f31bfffd98c174858d31b95d88607046a07314f6e964b"
CLOSURE_BOUNDARY = "generated-attributes-transitive-closure-and-core-scope-configuration"
BOUNDARY_EXAMPLES = ("include::{generated}/specattribs.adoc[]", "include::{chapters}/VK_KHR_surface/wsi.adoc[]",
                     "include::{chapters}/extensions.adoc[]")


class IncludeError(ValueError):
    """A Vulkan specification include closure is unsafe or stale."""


def reject(message: str) -> None:
    raise IncludeError(message)


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


CANDIDATES = reviewed_module("f024_candidate_contract_for_vulkan_spec", PARENT / "candidate_contract.py")


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"specification includes cannot be read: {error}")
    if not isinstance(value, dict):
        reject("specification includes are not a JSON object")
    return value


def rejected_candidate(profile: str, candidates: Path) -> None:
    identity = CANDIDATES.CATALOG.get(profile, {}).get(CANDIDATE_ID)
    if not isinstance(identity, dict) or identity.get("sha256") != CANDIDATE_SHA256 or identity.get("decision") != "rejected":
        reject("specification includes do not bind the reviewed rejected root")
    try:
        requirements, decisions = CANDIDATES.requirements(profile), CANDIDATES.validate(candidates, profile)
    except CANDIDATES.AuditError as error:
        reject(f"specification candidate contract failed: {error}")
    matching = [decision for (identifier, role), decision in zip(requirements, decisions)
                if identifier == CANDIDATE_ID and role == "api-limit-format-spec"]
    if matching != ["rejected"]:
        reject("specification includes do not bind a rejected normative candidate")


def include_list(value: object, digest: object, count: object) -> tuple[str, ...]:
    if not isinstance(value, list) or type(count) is not int or count != INCLUDE_COUNT or digest != INCLUDE_SHA256:
        reject("specification includes have a stale count or digest")
    if len(value) != count or any(not isinstance(item, str) for item in value):
        reject("specification includes have an incorrect count")
    result = tuple(value)
    for item in result:
        if not item.startswith("include::") or not item.endswith("[]") or "\n" in item or "\r" in item:
            reject("specification include syntax is unsafe")
        target = item.removeprefix("include::").removesuffix("[]")
        path = PurePosixPath(target)
        if not target or path.is_absolute() or str(path) != target or ".." in path.parts or not target.endswith(".adoc"):
            reject("specification include path is unsafe")
    if len(set(result)) != len(result):
        reject("specification includes repeat a path")
    rendered = ("\n".join(result) + "\n").encode("utf-8")
    if hashlib.sha256(rendered).hexdigest() != digest:
        reject("specification includes do not reproduce the reviewed directive transcript")
    return result


def validate(path: Path, candidates: Path, profile: str) -> int:
    rejected_candidate(profile, candidates)
    value = document(path)
    if set(value) != ROOT_FIELDS or type(value.get("schema")) is not int or value["schema"] != 1 or value.get("profile") != profile:
        reject("specification includes do not match schema version 1")
    if value.get("candidate_id") != CANDIDATE_ID or value.get("candidate_sha256") != CANDIDATE_SHA256:
        reject("specification includes do not bind the reviewed root")
    result = include_list(value.get("direct_includes"), value.get("include_list_sha256"), value.get("direct_include_count"))
    examples = value.get("boundary_examples")
    if value.get("closure_boundary") != CLOSURE_BOUNDARY or not isinstance(examples, list) or tuple(examples) != BOUNDARY_EXAMPLES:
        reject("specification includes do not bind the closure boundary")
    if not set(examples).issubset(result):
        reject("specification boundary examples are absent from the reviewed root")
    return len(result)


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit("usage: spec_include_contract.py INCLUDES.json CANDIDATES.json PROFILE")
    try:
        count = validate(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3])
    except IncludeError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"INCLUDES: {sys.argv[3]} {count} direct specification includes")


if __name__ == "__main__":
    main()
