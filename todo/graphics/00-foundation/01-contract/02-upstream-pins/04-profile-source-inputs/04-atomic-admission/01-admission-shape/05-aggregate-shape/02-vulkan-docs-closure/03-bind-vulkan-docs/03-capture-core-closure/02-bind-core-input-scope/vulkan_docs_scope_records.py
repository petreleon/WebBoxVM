"""Validate normalized observer inputs before any core-scope interpretation."""

from __future__ import annotations

from vulkan_docs_scope_model import (
    DERIVED, INCLUDE_FIELDS, MAX_IGNORED_READS, MAX_INCLUDES, MAX_RECORDS, NORMALIZED_FIELDS, PHASES, RAW,
    RECORD_FIELDS, CaptureExpectation, reject,
)
from vulkan_docs_scope_parse import canonical, digest, exact_object, nonnegative, phases, positive, selector, source_limit


def record(value: object) -> dict[str, object]:
    row = exact_object(value, RECORD_FIELDS, "observed input")
    kind = row.get("kind")
    if kind not in (RAW, DERIVED):
        reject("observed input has an unknown kind")
    name = selector(row.get("selector"), "observed input selector")
    if any(character in name for character in "?#%"):
        reject("observed input selector has URL-reserved characters")
    if name == "out" or name.startswith("out/") or name == "generated/out" or name.startswith("generated/out/"):
        reject("rendered output cannot be a scope input")
    if (kind == RAW and name.startswith("generated/")) or (kind == DERIVED and not name.startswith("generated/")):
        reject("observed input kind does not match its mounted selector namespace")
    roles = phases(row.get("phase_roles"), "observed input phases")
    if not set(roles) <= PHASES:
        reject("observed input has an unknown phase")
    return {"kind": kind, "selector": name, "sha256": digest(row.get("sha256"), "observed input sha256"),
            "bytes": source_limit(row.get("bytes"), "observed input bytes"), "phase_roles": list(roles)}


def include(value: object, index: dict[tuple[str, str], dict[str, object]]) -> dict[str, object]:
    row = exact_object(value, INCLUDE_FIELDS, "resolved include")
    kind = row.get("kind")
    if kind not in (RAW, DERIVED):
        reject("resolved include has an unknown kind")
    name = selector(row.get("selector"), "resolved include selector")
    target = index.get((kind, name))
    if target is None or "asciidoctor" not in target["phase_roles"]:
        reject("resolved include has no matching Asciidoctor input")
    return {"kind": kind, "selector": name, "line": positive(row.get("line"), "resolved include line", MAX_INCLUDES)}


def normalized(value: object, expected: CaptureExpectation) -> tuple[tuple[dict[str, object], ...], tuple[dict[str, object], ...]]:
    data = exact_object(value, NORMALIZED_FIELDS, "normalized observer input")
    rows = data.get("records")
    if not isinstance(rows, list) or not rows or len(rows) > MAX_RECORDS:
        reject("normalized observer has an invalid input count")
    parsed = tuple(record(item) for item in rows)
    keys = tuple((item["kind"], item["selector"]) for item in parsed)
    if keys != tuple(sorted(keys)) or len(set(keys)) != len(keys) or len({key[1] for key in keys}) != len(keys):
        reject("normalized observer inputs are not ordered unique identities")
    index = dict(zip(keys, parsed, strict=True))
    includes = data.get("includes")
    if not isinstance(includes, list) or not includes or len(includes) > MAX_INCLUDES:
        reject("normalized observer has an invalid include count")
    resolved = tuple(include(item, index) for item in includes)
    raw_count = sum(item["kind"] == RAW for item in parsed)
    derived_count = len(parsed) - raw_count
    if (positive(data.get("raw_count"), "normalized raw count", MAX_RECORDS),
            positive(data.get("derived_count"), "normalized derived count", MAX_RECORDS)) != (raw_count, derived_count):
        reject("normalized observer has stale input counts")
    if (raw_count, derived_count, len(resolved)) != (expected.raw_count, expected.derived_count, expected.include_count):
        reject("normalized observer does not match its selected observation run")
    nonnegative(data.get("ignored_runtime_reads"), "normalized ignored reads", MAX_IGNORED_READS)
    phase_counts = data.get("phase_counts")
    if not isinstance(phase_counts, dict) or set(phase_counts) != PHASES:
        reject("normalized observer has an invalid phase-count schema")
    actual_phases = {name: sum(name in item["phase_roles"] for item in parsed) for name in sorted(PHASES)}
    if phase_counts != actual_phases or phase_counts != expected.phase_counts or any(count < 1 for count in actual_phases.values()):
        reject("normalized observer has stale or incomplete phase coverage")
    manifest = canonical(list(parsed), "webboxvm-graphics-vulkan-docs-observed-input-manifest-v1")
    include_digest = canonical(list(resolved), "webboxvm-graphics-vulkan-docs-resolved-includes-v1")
    if (digest(data.get("input_manifest_sha256"), "normalized input manifest"),
            digest(data.get("include_identity_sha256"), "normalized include identity")) != (manifest, include_digest):
        reject("normalized observer has a stale identity")
    if (manifest, include_digest) != (expected_manifest(expected), expected_include(expected)):
        reject("normalized observer identity does not match the selected run")
    return parsed, resolved


def expected_manifest(expected: CaptureExpectation) -> str:
    return getattr(expected, "input_manifest_digest")


def expected_include(expected: CaptureExpectation) -> str:
    return getattr(expected, "include_digest")
