#!/usr/bin/env python3
"""Fail-closed pre-admission map for every frozen F03 source shape."""

from __future__ import annotations

import hashlib, json, sys
from dataclasses import dataclass
from pathlib import Path
from source_map_inputs import (AUDITS, CANDIDATES, EXPECTED, GLES, GLES_CLOSURE, GLES_CONFIGURATIONS,
                               INCLUDES, PROFILE, REFERENCES, SOURCE_MAP, VULKAN_INCLUDES, VULKAN_REFERENCES)
COMMON = frozenset(("profile", "role", "required_input_id", "shape", "state", "root_sha256", "root_selector"))
MEMBER = frozenset(("id", "sha256", "selector"))
DIRECT = COMMON
CLOSURE = COMMON | frozenset(("core_members", "excluded_members", "configuration_sha256",
                              "core_configuration_count", "core_case_count", "core_case_configuration_runs",
                              "excluded_configuration_count", "blocker"))
UNRESOLVED = COMMON | frozenset(("blocker", "observation"))

class SourceMapError(ValueError):
    """A logical source shape is incomplete, stale, or falsely admitted."""

@dataclass(frozen=True)
class ShapeRecord:
    profile: str
    role: str
    required_input_id: str
    shape: str
    root_sha256: str
    state: str
    member_count: int = 0
    configuration_count: int = 0
    observation_count: int = 0

def reject(message: str) -> None:
    raise SourceMapError(message)

def document(path: Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"{label} cannot be read: {error}")
    if not isinstance(value, dict):
        reject(f"{label} is not a JSON object")
    return value

def audited(paths: dict[str, Path]) -> dict[tuple[str, str], dict[str, object]]:
    if not isinstance(paths, dict) or set(paths) != {profile for profile, _, _ in EXPECTED} or any(not isinstance(path, Path) for path in paths.values()):
        reject("source map has an incomplete candidate-audit path set")
    result = {}
    for profile, path in paths.items():
        try:
            expected, decisions = CANDIDATES.requirements(profile), CANDIDATES.validate(path, profile)
        except CANDIDATES.AuditError as error:
            reject(f"candidate audit failed: {error}")
        records = CANDIDATES.document(path).get("candidates")
        if not isinstance(records, list) or len(records) != len(expected) or len(decisions) != len(expected):
            reject("candidate audit has an invalid reviewed record count")
        for requirement, decision, record in zip(expected, decisions, records):
            if not isinstance(record, dict) or (record.get("required_input_id"), record.get("role")) != requirement:
                reject("candidate audit record does not retain its required role")
            if record.get("decision") != decision:
                reject("candidate audit record has a stale decision")
            result[(profile, requirement[0])] = record
    return result

def common(value: object, expected: tuple[str, str, str], candidate: dict[str, object], fields: frozenset[str]) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject("source shape has an unexpected schema")
    if tuple(value.get(field) for field in ("profile", "role", "required_input_id")) != expected:
        reject("source shape does not retain canonical requirement order")
    entry = candidate.get("entry")
    if not isinstance(entry, dict) or value.get("root_sha256") != entry.get("sha256") or value.get("root_selector") != candidate.get("selector"):
        reject("source shape does not bind its reviewed root identity")
    return value


def members(value: object) -> tuple[tuple[str, str, str], ...]:
    if not isinstance(value, list):
        reject("bounded closure has malformed physical members")
    result = []
    for item in value:
        if not isinstance(item, dict) or set(item) != MEMBER or any(not isinstance(item[field], str) for field in MEMBER):
            reject("bounded closure has malformed physical member identity")
        result.append((item["id"], item["sha256"], item["selector"]))
    return tuple(result)


def direct(value: object, expected: tuple[str, str, str], candidate: dict[str, object]) -> ShapeRecord:
    value = common(value, expected, candidate, DIRECT)
    if (value.get("shape"), value.get("state"), candidate.get("decision"), candidate.get("coverage"), candidate.get("admission_blocker")) != ("complete-single-source", "candidate-accepted", "accepted", "complete-single-file", ""):
        reject("complete source has an invalid admission shape")
    return ShapeRecord(*expected, "complete-single-source", str(value["root_sha256"]), "candidate-accepted")


def bounded(value: object, expected: tuple[str, str, str], candidate: dict[str, object], audits: dict[str, Path], closure_path: Path, configurations_path: Path) -> ShapeRecord:
    value = common(value, expected, candidate, CLOSURE)
    if (value.get("shape"), value.get("state"), candidate.get("decision"), candidate.get("coverage"), value.get("blocker")) != ("bounded-unadmitted-closure", "unadmitted", "rejected", "compound-unadmitted", candidate.get("admission_blocker")):
        reject("bounded closure has an invalid admission state")
    try:
        shape = GLES.validate(closure_path, configurations_path, audits["gles-3.2"], expected[0])
    except GLES.ShapeError as error:
        reject(f"GLES closure probe failed: {error}")
    closure, configurations = document(closure_path, "GLES closure"), document(configurations_path, "GLES configurations")
    core, excluded = members(value.get("core_members")), members(value.get("excluded_members"))
    actual_core = tuple((item["id"], item["sha256"], item["selector"]) for item in closure["included"])
    actual_excluded = tuple((item["id"], item["sha256"], item["selector"]) for item in closure["excluded"])
    expected_counts = (configurations["core_configuration_count"], closure["core_case_count"], configurations["core_case_configuration_runs"], configurations["excluded_configuration_count"])
    counts = tuple(value.get(field) for field in ("core_configuration_count", "core_case_count", "core_case_configuration_runs", "excluded_configuration_count"))
    if core != actual_core or excluded != actual_excluded or counts != expected_counts or any(type(count) is not int for count in counts) or shape.configuration_count != counts[0]:
        reject("bounded closure does not bind its exact members or configuration scope")
    if value.get("configuration_sha256") != hashlib.sha256(configurations_path.read_bytes()).hexdigest():
        reject("bounded closure has a stale configuration identity")
    return ShapeRecord(*expected, "bounded-unadmitted-closure", str(value["root_sha256"]), "unadmitted", len(core), counts[0])


def unresolved(value: object, expected: tuple[str, str, str], candidate: dict[str, object], audits: dict[str, Path], includes_path: Path, references_path: Path) -> ShapeRecord:
    value = common(value, expected, candidate, UNRESOLVED)
    identifier = expected[2]
    if (value.get("shape"), value.get("state"), candidate.get("decision"), candidate.get("coverage"), value.get("blocker")) != ("unresolved-root", "unadmitted", "rejected", "compound-unadmitted", candidate.get("admission_blocker")):
        reject("unresolved root has an invalid admission state")
    try:
        if identifier == "vulkan-14-spec":
            count, detail = INCLUDES.validate(includes_path, audits["vulkan-1.4-core"], expected[0]), ("direct-include-transcript", INCLUDES.INCLUDE_SHA256, INCLUDES.CLOSURE_BOUNDARY, INCLUDES.BOUNDARY_EXAMPLES)
        else:
            count, detail = REFERENCES.validate(references_path, audits["vulkan-1.4-core"], expected[0]), ("direct-reference-transcript", candidate["entry"]["sha256"], REFERENCES.SCOPE_BOUNDARY, REFERENCES.SCOPE_EXAMPLES)
    except (INCLUDES.IncludeError, REFERENCES.ReferenceError) as error:
        reject(f"unresolved-root observation failed: {error}")
    observation = value.get("observation")
    examples = observation.get("examples") if isinstance(observation, dict) else None
    if (not isinstance(observation, dict) or set(observation) != {"kind", "count", "sha256", "boundary", "examples"}
            or type(observation.get("count")) is not int or tuple(observation.get(field) for field in ("kind", "sha256", "boundary")) != detail[:3]
            or not isinstance(examples, list) or tuple(examples) != detail[3] or observation["count"] != count):
        reject("unresolved root does not bind its reviewed boundary observation")
    return ShapeRecord(*expected, "unresolved-root", str(value["root_sha256"]), "unadmitted", observation_count=count)


def validate(path: Path = SOURCE_MAP, audits: dict[str, Path] = AUDITS, closure_path: Path = GLES_CLOSURE,
             configurations_path: Path = GLES_CONFIGURATIONS, includes_path: Path = VULKAN_INCLUDES,
             references_path: Path = VULKAN_REFERENCES) -> tuple[ShapeRecord, ...]:
    if not isinstance(path, Path) or any(not isinstance(item, Path) for item in (closure_path, configurations_path, includes_path, references_path)):
        reject("source map has an invalid auxiliary input path")
    try:
        PROFILE.validate()
    except PROFILE.ScopeError as error:
        reject(f"canonical F03 requirements are invalid: {error}")
    source = document(path, "source map")
    records = source.get("shapes")
    if set(source) != {"schema", "shapes"} or type(source.get("schema")) is not int or source.get("schema") != 1 or not isinstance(records, list):
        reject("source map does not match schema version 1")
    if tuple((item.get("profile"), item.get("role"), item.get("required_input_id")) if isinstance(item, dict) else () for item in records) != EXPECTED:
        reject("source map does not exactly enumerate the six canonical requirements")
    reviewed = audited(audits)
    result = []
    for item, expected in zip(records, EXPECTED):
        candidate = reviewed.get((expected[0], expected[2]))
        if candidate is None:
            reject("source map has no reviewed candidate root")
        if expected[2] in {"opengl-46-core-spec", "opengl-cts-manifest", "gles-32-spec"}:
            result.append(direct(item, expected, candidate))
        elif expected[2] == "gles-cts-manifest":
            result.append(bounded(item, expected, candidate, audits, closure_path, configurations_path))
        else:
            result.append(unresolved(item, expected, candidate, audits, includes_path, references_path))
    return tuple(result)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: source_map_contract.py SOURCE_MAP.json")
    try:
        records = validate(Path(sys.argv[1]))
    except SourceMapError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    totals = tuple(sum(record.shape == kind for record in records) for kind in ("complete-single-source", "bounded-unadmitted-closure", "unresolved-root"))
    print(f"MAP: {len(records)} required inputs, {totals[0]} complete single-source, {totals[1]} unadmitted closure, {totals[2]} unresolved roots")


if __name__ == "__main__":
    main()
