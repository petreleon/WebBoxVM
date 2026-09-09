#!/usr/bin/env python3
"""Fail-closed transition design that can only report today's pre-admission state."""

from __future__ import annotations

import importlib.util
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))
from post_cutover_schema import (ATOMIC_CUTOVER, CANDIDATE_AUDIT, CONSUMERS, FUTURE_ARTIFACTS,
                                 SOURCE_SHAPES)
ROOT = HERE.parents[2]
RULES = HERE / "post_cutover_rules.json"
MAP_DIR, BOUNDARY_DIR = HERE.parent / "02-source-map", HERE.parent / "03-vulkan-boundaries"
if str(MAP_DIR) not in sys.path:
    sys.path.insert(0, str(MAP_DIR))
ROOT_FIELDS = frozenset(("schema", "phase", "source_shapes", "candidate_audit", "atomic_cutover", "future_artifacts", "renew_consumers"))
CURRENT = (
    ("opengl-46-core-spec", "complete-single-source", "candidate-accepted"),
    ("opengl-cts-manifest", "complete-single-source", "candidate-accepted"),
    ("gles-32-spec", "complete-single-source", "candidate-accepted"),
    ("gles-cts-manifest", "bounded-unadmitted-closure", "unadmitted"),
    ("vulkan-14-spec", "unresolved-root", "unadmitted"),
    ("vulkan-cts-mustpass", "unresolved-root", "unadmitted"),
)
BLOCKERS = ("requires-multifile-core-selector-closure", "vulkan-docs-core-generated-closure-unadmitted",
            "vcts-vk-default-compound-oversize-core-scope-unadmitted")


class TransitionError(ValueError):
    """A design input could permit a partial or root-only cutover."""


@dataclass(frozen=True)
class PreAdmission:
    required_input_ids: tuple[str, ...]
    inventory_revision: str
    blockers: tuple[str, ...]
    state: str = field(default="pre-admission", init=False)
    cutover_ready: bool = field(default=False, init=False)


def reject(message: str) -> None:
    raise TransitionError(message)


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            reject("transition design has duplicate JSON object fields")
        value[key] = item
    return value


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


MAP = reviewed_module("f024_transition_map", MAP_DIR / "source_map_contract.py")
BOUNDARY = reviewed_module("f024_transition_boundary", BOUNDARY_DIR / "boundary_contract.py")
LAYOUT = reviewed_module("f024_transition_inventory", ROOT.parent / "01-input-inventory/inventory_layout.py")
MANIFEST = ROOT.parent / "01-input-inventory/manifest.toml"


def document(path: Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"{label} cannot be read: {error}")
    if not isinstance(value, dict):
        reject(f"{label} is not a JSON object")
    return value


def audit_bundle(audits: object) -> dict[str, Path]:
    if (not isinstance(audits, dict) or set(audits) != set(MAP.AUDITS)
            or any(not isinstance(path, Path) for path in audits.values())):
        reject("transition design has an incomplete candidate-audit bundle")
    for profile, path in audits.items():
        document(path, f"{profile} candidate audit")
    return dict(audits)


def current(source_map_path: Path, audits: dict[str, Path]) -> tuple[tuple[str, ...], tuple[str, ...]]:
    source = document(source_map_path, "source map")
    try:
        records = MAP.validate(source_map_path, audits)
    except MAP.SourceMapError as error:
        reject(f"source map failed: {error}")
    actual = tuple((record.required_input_id, record.shape, record.state) for record in records)
    if actual != CURRENT:
        reject("source map does not retain the canonical pre-admission source states")
    shapes = source.get("shapes")
    blockers = tuple(item.get("blocker") for item in shapes if isinstance(item, dict) and item.get("state") == "unadmitted") if isinstance(shapes, list) else ()
    if blockers != BLOCKERS:
        reject("source map does not retain all compound-source blockers")
    return tuple(item[0] for item in actual), blockers


def inventory_revision(audits: dict[str, Path]) -> str:
    try:
        revision = LAYOUT.load_inventory(MANIFEST).revision
    except LAYOUT.InventoryLayoutError as error:
        reject(f"current inventory cannot be loaded: {error}")
    if not isinstance(audits, dict) or any(document(path, "candidate audit").get("inventory_sha256") != revision for path in audits.values()):
        reject("candidate audit has a stale or mixed inventory lock")
    return revision


def boundaries(path: Path, source_map_path: Path, audits: dict[str, Path]) -> None:
    document(path, "Vulkan boundaries")
    try:
        records = BOUNDARY.validate(path, source_map_path, audits)
    except BOUNDARY.BoundaryError as error:
        reject(f"Vulkan boundary failed: {error}")
    if tuple((item.required_input_id, item.state, item.observation_count) for item in records) != (("vulkan-14-spec", "unadmitted", 73), ("vulkan-cts-mustpass", "unadmitted", 98)):
        reject("Vulkan roots are not retained as unresolved pre-admission boundaries")


def policy(path: Path) -> None:
    value = document(path, "transition rules")
    if (set(value) != ROOT_FIELDS or type(value.get("schema")) is not int or value.get("schema") != 1
            or value.get("phase") != "transition-design" or value.get("source_shapes") != list(SOURCE_SHAPES)
            or value.get("candidate_audit") != CANDIDATE_AUDIT or value.get("atomic_cutover") != ATOMIC_CUTOVER
            or value.get("future_artifacts") != list(FUTURE_ARTIFACTS) or value.get("renew_consumers") != list(CONSUMERS)):
        reject("transition rules do not define the only valid atomic cutover")


def validate(path: Path = RULES, source_map_path: Path = MAP.SOURCE_MAP,
             boundary_path: Path = BOUNDARY.BOUNDARIES, audits: dict[str, Path] = MAP.AUDITS) -> PreAdmission:
    if not all(isinstance(item, Path) for item in (path, source_map_path, boundary_path)):
        reject("transition design has an invalid input path")
    bundle = audit_bundle(audits)
    policy(path)
    inputs, blockers = current(source_map_path, bundle)
    boundaries(boundary_path, source_map_path, bundle)
    return PreAdmission(inputs, inventory_revision(bundle), blockers)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: post_cutover_contract.py RULES.json")
    try:
        result = validate(Path(sys.argv[1]))
    except TransitionError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"TRANSITION: {result.state}, {len(result.required_input_ids)} required inputs, 0 cutover-ready")


if __name__ == "__main__":
    main()
