#!/usr/bin/env python3
"""Fail-closed pre-admission shape for a compound logical source closure."""

from __future__ import annotations

import importlib.util
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path

HERE = Path(__file__).resolve().parent
INPUT_FIELDS = ("id", "source_family", "immutable_url", "revision", "sha256", "bytes", "license",
                "local_cache", "generated_code_role", "provenance")


class ShapeError(ValueError):
    """A pre-admission logical closure is incomplete, stale, or unsafe."""


@dataclass(frozen=True)
class UnadmittedClosure:
    profile: str
    required_input_id: str
    root_sha256: str
    core_member_ids: tuple[str, ...]
    excluded_member_ids: tuple[str, ...]
    configuration_count: int
    state: str = field(default="unadmitted", init=False)


def reject(message: str) -> None:
    raise ShapeError(message)


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


COMPOUND = reviewed_module("f024_admission_compound", HERE.parent.parent / "compound_selector_contract.py")


def document(path: Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"{label} cannot be read: {error}")
    if not isinstance(value, dict):
        reject(f"{label} is not a JSON object")
    return value


def physical_member(value: object, reason: str) -> tuple[str, str, str]:
    if not isinstance(value, dict) or set(value) != COMPOUND.COMPONENT_FIELDS:
        reject("logical closure has an invalid physical-member schema")
    if value.get("reason") != reason or type(value.get("case_count")) is not int or value["case_count"] < 1:
        reject("logical closure has an invalid member role or case count")
    try:
        source = COMPOUND.CANDIDATES.SourceInput.from_manifest({field: value[field] for field in INPUT_FIELDS})
        COMPOUND.selector(value["selector"], value["immutable_url"])
    except (COMPOUND.CANDIDATES.ContractError, COMPOUND.CompoundError) as error:
        reject(f"logical closure member violates F02.2 policy: {error}")
    identifier, selector = value.get("id"), value.get("selector")
    if not isinstance(identifier, str) or not identifier or not isinstance(selector, str):
        reject("logical closure member has an invalid id or selector")
    return identifier, str(source.local_cache), selector


def members(included: object, excluded: object) -> tuple[tuple[str, ...], tuple[str, ...]]:
    if not isinstance(included, list) or not isinstance(excluded, list) or not included or not excluded:
        reject("logical closure needs core members and an explicit exclusion boundary")
    all_items = included + excluded
    for field in ("id", "local_cache", "selector"):
        values = [item.get(field) if isinstance(item, dict) else None for item in all_items]
        if all(isinstance(value, str) for value in values) and len(set(values)) != len(values):
            reject(f"logical closure repeats a physical member {field}")
    core = tuple(physical_member(item, "core") for item in included)
    boundary = tuple(physical_member(item, "optional-extension") for item in excluded)
    if set(member[2] for member in core) & set(member[2] for member in boundary):
        reject("logical closure mixes a core selector with an excluded selector")
    return tuple(member[0] for member in core), tuple(member[0] for member in boundary)


def validate(closure_path: Path, configurations_path: Path, candidates_path: Path, profile: str) -> UnadmittedClosure:
    try:
        selectors, _, configurations, _, excluded, _ = COMPOUND.validate(
            closure_path, configurations_path, candidates_path, profile)
    except COMPOUND.CompoundError as error:
        reject(f"compound audit failed: {error}")
    closure = document(closure_path, "logical closure")
    core, boundary = members(closure.get("included"), closure.get("excluded"))
    if len(core) != selectors or len(boundary) != excluded:
        reject("logical closure count does not match its compound audit")
    candidate_id, root = closure.get("candidate_id"), closure.get("candidate_sha256")
    if not isinstance(candidate_id, str) or not isinstance(root, str):
        reject("logical closure does not bind its rejected root")
    return UnadmittedClosure(profile, candidate_id, root, core, boundary, configurations)


def main() -> None:
    if len(sys.argv) != 5:
        raise SystemExit("usage: closure_shape_contract.py CLOSURE.json CONFIGURATIONS.json CANDIDATES.json PROFILE")
    try:
        shape = validate(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]), sys.argv[4])
    except ShapeError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"SHAPE: {shape.profile} {shape.state}, {len(shape.core_member_ids)} core members, "
          f"{len(shape.excluded_member_ids)} excluded members, {shape.configuration_count} configurations")


if __name__ == "__main__":
    main()
