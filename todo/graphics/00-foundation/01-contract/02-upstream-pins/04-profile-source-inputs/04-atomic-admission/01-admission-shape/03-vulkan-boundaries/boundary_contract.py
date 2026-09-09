#!/usr/bin/env python3
"""Fail-closed pre-admission boundaries for the two unresolved Vulkan roots."""

from __future__ import annotations

import importlib.util
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
BOUNDARIES = HERE / "boundaries.json"
MAP_DIR = HERE.parent / "02-source-map"
if str(MAP_DIR) not in sys.path:
    sys.path.insert(0, str(MAP_DIR))
ROOT_FIELDS = frozenset(("schema", "profile", "boundaries"))
COMMON = frozenset(("profile", "role", "required_input_id", "shape", "state", "root_sha256",
                    "blocker", "root_fallback", "admission_requirements", "scope_exclusions", "observation"))
VCTS = COMMON | frozenset(("oversize_inspection",))
INSPECTION = frozenset(("member_identities", "max_input_bytes", "inspected_over_limit_member_count",
                        "inspected_member_total_bytes"))
SCOPES = ("wsi", "video", "extensions")
DOCS = ("vulkan-14-spec", "api-limit-format-spec",
        ("generated-and-transitive-members-pinned", "macro-and-conditional-configuration-pinned", "core-only-scope-bounded"),
        (("wsi", ("include::{chapters}/VK_KHR_surface/wsi.adoc[]",)), ("video", ("include::{chapters}/videocoding.adoc[]",)),
         ("extensions", ("include::{chapters}/extensions.adoc[]",))))
MUSTPASS = ("vulkan-cts-mustpass", "conformance-manifest",
            ("selected-member-identities-and-hashes-pinned", "recursive-dependencies-pinned", "core-only-selection-bounded"),
            (("wsi", ("vk-default/wsi.txt",)), ("video", ("vk-default/video.txt",)),
             ("extensions", ("vk-default/ray-query.txt", "vk-default/cooperative-vector.txt", "vk-default/data-graph.txt", "vk-default/tensor.txt"))))
EXPECTED = (DOCS, MUSTPASS)


class BoundaryError(ValueError):
    """A Vulkan root has been falsely presented as an admissible closure."""


@dataclass(frozen=True)
class UnadmittedBoundary:
    required_input_id: str
    observation_count: int
    state: str = field(default="unadmitted", init=False)


def reject(message: str) -> None:
    raise BoundaryError(message)


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            reject("Vulkan boundary has duplicate JSON object fields")
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


MAP = reviewed_module("f024_vulkan_boundary_map", MAP_DIR / "source_map_contract.py")
MODEL = reviewed_module("f024_vulkan_boundary_model", ROOT.parent / "02-fetch-verifier/01-fetch-contract/source_model.py")


def document(path: Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"{label} cannot be read: {error}")
    if not isinstance(value, dict):
        reject(f"{label} is not a JSON object")
    return value


def mapped(path: Path) -> dict[str, dict[str, object]]:
    try:
        records = MAP.validate(path)
    except MAP.SourceMapError as error:
        reject(f"source map failed: {error}")
    source = document(path, "source map")
    shapes = source.get("shapes")
    rows = tuple((record, item) for record, item in zip(records, shapes) if record.profile == "vulkan-1.4-core") if isinstance(shapes, list) else ()
    expected = tuple((item[0], "unresolved-root", "unadmitted") for item in EXPECTED)
    actual = tuple((record.required_input_id, record.shape, record.state) for record, _ in rows)
    if actual != expected or any(not isinstance(item, dict) for _, item in rows):
        reject("source map does not retain two unadmitted unresolved Vulkan roots")
    return {record.required_input_id: item for record, item in rows}


def transcript(identifier: str) -> set[str]:
    path, field_name = ((MAP.VULKAN_INCLUDES, "direct_includes") if identifier == DOCS[0]
                        else (MAP.VULKAN_REFERENCES, "direct_references"))
    values = document(path, "reviewed Vulkan transcript").get(field_name)
    if not isinstance(values, list) or any(not isinstance(item, str) for item in values):
        reject("reviewed Vulkan transcript has malformed selectors")
    return set(values)


def exclusions(value: object, expected: tuple[tuple[str, tuple[str, ...]], ...], selectors: set[str]) -> None:
    if not isinstance(value, dict) or tuple(value) != SCOPES:
        reject("Vulkan boundary has an invalid scope-exclusion schema")
    actual = tuple((scope, tuple(value[scope]) if isinstance(value[scope], list) else ()) for scope in SCOPES)
    if actual != expected or any(selector not in selectors for _, values in actual for selector in values):
        reject("Vulkan boundary erases or expands a reviewed scope exclusion")


def oversize(value: object) -> None:
    expected = ("unpinned", MODEL.MAX_INPUT_BYTES, 14, 434669348)
    if (not isinstance(value, dict) or set(value) != INSPECTION
            or tuple(value.get(field) for field in ("member_identities", "max_input_bytes", "inspected_over_limit_member_count", "inspected_member_total_bytes")) != expected
            or value["inspected_member_total_bytes"] <= value["max_input_bytes"]):
        reject("VCTS oversize inspection is incomplete or weakened")


def boundary(value: object, source: dict[str, object], expected: tuple[object, ...]) -> UnadmittedBoundary:
    identifier, role, requirements, scope = expected
    fields = VCTS if identifier == MUSTPASS[0] else COMMON
    if not isinstance(value, dict) or set(value) != fields:
        reject("Vulkan boundary has an unexpected schema")
    if tuple(value.get(field) for field in ("profile", "role", "required_input_id", "shape", "state", "root_fallback")) != ("vulkan-1.4-core", role, identifier, "unresolved-root", "unadmitted", "forbidden"):
        reject("Vulkan boundary has an invalid pre-admission state")
    if (value.get("root_sha256"), value.get("blocker"), value.get("observation")) != (source.get("root_sha256"), source.get("blocker"), source.get("observation")):
        reject("Vulkan boundary does not bind its audited root and observation")
    if value.get("admission_requirements") != list(requirements):
        reject("Vulkan boundary does not retain every admission requirement")
    exclusions(value.get("scope_exclusions"), scope, transcript(str(identifier)))
    if identifier == MUSTPASS[0]:
        oversize(value.get("oversize_inspection"))
    observation = value["observation"]
    return UnadmittedBoundary(str(identifier), int(observation["count"]))


def validate(path: Path = BOUNDARIES, source_map_path: Path = MAP.SOURCE_MAP) -> tuple[UnadmittedBoundary, ...]:
    if not isinstance(path, Path) or not isinstance(source_map_path, Path):
        reject("Vulkan boundary has an invalid input path")
    source = mapped(source_map_path)
    value = document(path, "Vulkan boundaries")
    records = value.get("boundaries")
    if set(value) != ROOT_FIELDS or type(value.get("schema")) is not int or value.get("schema") != 1 or value.get("profile") != "vulkan-1.4-core" or not isinstance(records, list):
        reject("Vulkan boundaries do not match schema version 1")
    if tuple(item.get("required_input_id") if isinstance(item, dict) else None for item in records) != tuple(item[0] for item in EXPECTED):
        reject("Vulkan boundaries do not enumerate the two canonical roots")
    return tuple(boundary(item, source[str(expected[0])], expected) for item, expected in zip(records, EXPECTED))


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: boundary_contract.py BOUNDARIES.json")
    try:
        records = validate(Path(sys.argv[1]))
    except BoundaryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"BOUNDARIES: {len(records)} unresolved Vulkan roots, 0 admitted closures")


if __name__ == "__main__":
    main()
