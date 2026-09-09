#!/usr/bin/env python3
"""Compose reviewed source facts into an immutable, blocked pre-admission aggregate."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SHAPE = HERE.parent.parent
TRANSITION_DIR = SHAPE / "04-post-cutover-rules"
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))
if str(TRANSITION_DIR) not in sys.path:
    sys.path.insert(0, str(TRANSITION_DIR))
from aggregate_model import BLOCKERS, DIRECT, GLES_COUNTS, VULKAN
from aggregate_model import GlesClosure, PreAdmissionAggregate, VulkanObservation

class AggregateError(ValueError):
    """A current source fact is incomplete or falsely presented as ready."""


def reject(message: str) -> None:
    raise AggregateError(message)

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

TRANSITION = reviewed_module("f024_pre_admission_transition", TRANSITION_DIR / "post_cutover_contract.py")

def document(path: Path, label: str) -> dict[str, object]:
    try:
        return TRANSITION.document(path, label)
    except TRANSITION.TransitionError as error:
        reject(str(error))

def identities(value: object, label: str) -> tuple[tuple[str, str, str], ...]:
    if not isinstance(value, list):
        reject(f"{label} has no immutable member list")
    rows = []
    for item in value:
        if not isinstance(item, dict) or set(item) != {"id", "sha256", "selector"}:
            reject(f"{label} has a malformed immutable member")
        row = tuple(item.get(name) for name in ("id", "sha256", "selector"))
        if any(not isinstance(part, str) or not part for part in row):
            reject(f"{label} has an incomplete immutable member")
        rows.append(row)
    if len({row[0] for row in rows}) != len(rows) or len({row[1] for row in rows}) != len(rows):
        reject(f"{label} has a duplicate immutable member")
    return tuple(rows)

def gles(value: object) -> GlesClosure:
    if not isinstance(value, dict):
        reject("GLES closure is missing")
    core = identities(value.get("core_members"), "GLES core closure")
    excluded = identities(value.get("excluded_members"), "GLES exclusion boundary")
    count_names = (
        "core_configuration_count",
        "core_case_count",
        "core_case_configuration_runs",
        "excluded_configuration_count",
    )
    counts = tuple(value.get(name) for name in count_names)
    actual = (len(core), len(excluded), *counts)
    if actual != GLES_COUNTS or any(type(item) is not int for item in counts):
        reject("GLES closure does not retain its bounded member and configuration counts")
    digest = value.get("configuration_sha256")
    if not isinstance(digest, str) or len(digest) != 64:
        reject("GLES closure has no immutable configuration identity")
    return GlesClosure(core, excluded, digest, counts)

def vulkan(value: object, expected: tuple[str, int]) -> VulkanObservation:
    identifier, count = expected
    if not isinstance(value, dict) or "members" in value or value.get("root_fallback") != "forbidden":
        reject("Vulkan observation has a root substitute or fallback")
    observation, exclusions = value.get("observation"), value.get("scope_exclusions")
    state = (
        value.get("required_input_id"),
        value.get("state"),
        value.get("shape"),
    )
    if state != (identifier, "unadmitted", "unresolved-root"):
        reject("Vulkan observation has an invalid unresolved state")
    if not isinstance(observation, dict) or observation.get("count") != count:
        reject("Vulkan observation count is stale")
    if not isinstance(exclusions, dict) or tuple(exclusions) != ("wsi", "video", "extensions"):
        reject("Vulkan observation has an incomplete scope exclusion")
    scope = tuple((name, tuple(exclusions[name])) for name in exclusions)
    requirements = value.get("admission_requirements")
    if not isinstance(requirements, list) or any(not isinstance(item, str) or not item for item in requirements):
        reject("Vulkan observation has no closure requirements")
    root = value.get("root_sha256")
    if not isinstance(root, str) or len(root) != 64:
        reject("Vulkan observation has no immutable root identity")
    return VulkanObservation(identifier, root, count, tuple(requirements), scope)

def validate(
    rules_path: Path = TRANSITION.RULES,
    source_map_path: Path = TRANSITION.MAP.SOURCE_MAP,
    boundary_path: Path = TRANSITION.BOUNDARY.BOUNDARIES,
    audits: dict[str, Path] = TRANSITION.MAP.AUDITS,
) -> PreAdmissionAggregate:
    if not all(isinstance(path, Path) for path in (rules_path, source_map_path, boundary_path)):
        reject("aggregate has an invalid input path")
    try:
        pre = TRANSITION.validate(rules_path, source_map_path, boundary_path, audits)
    except TRANSITION.TransitionError as error:
        reject(f"transition failed: {error}")
    source, boundaries = document(source_map_path, "source map"), document(boundary_path, "Vulkan boundaries")
    shapes, boundary_rows = source.get("shapes"), boundaries.get("boundaries")
    if not isinstance(shapes, list) or not isinstance(boundary_rows, list):
        reject("aggregate has malformed reviewed records")
    source_ids = tuple(
        item.get("required_input_id") if isinstance(item, dict) else None
        for item in shapes
    )
    if source_ids != pre.required_input_ids:
        reject("aggregate does not retain exactly one canonical source shape per required ID")
    direct = tuple(
        item.get("required_input_id")
        for item in shapes
        if isinstance(item, dict) and item.get("shape") == "complete-single-source"
    )
    pairs = tuple(
        (item.get("required_input_id"), item.get("blocker"))
        for item in shapes
        if isinstance(item, dict) and item.get("state") == "unadmitted"
    )
    blockers = tuple(item[1] for item in BLOCKERS)
    if direct != DIRECT or pairs != BLOCKERS or tuple(pre.blockers) != blockers:
        reject("aggregate does not retain the exact current candidate and blocker partition")
    if len(shapes) != 6 or not isinstance(shapes[3], dict):
        reject("aggregate has no bounded GLES source shape")
    rows = tuple(vulkan(item, expected) for item, expected in zip(boundary_rows, VULKAN))
    vulkan_ids = tuple(item.required_input_id for item in rows)
    expected_ids = tuple(item[0] for item in VULKAN)
    if len(boundary_rows) != len(VULKAN) or vulkan_ids != expected_ids:
        reject("aggregate does not retain both unresolved Vulkan observations")
    return PreAdmissionAggregate(
        source_ids,
        direct,
        pairs,
        pre.inventory_revision,
        gles(shapes[3]),
        rows,
    )

def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: aggregate_contract.py RULES.json")
    try:
        result = validate(Path(sys.argv[1]))
    except AggregateError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    summary = (
        f"AGGREGATE: {result.state}, {len(result.required_input_ids)} required inputs, "
        f"{len(result.blockers)} blockers, 0 cutover-ready"
    )
    print(summary)

if __name__ == "__main__":
    main()
