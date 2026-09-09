#!/usr/bin/env python3
"""Fail closed around compact headers for observer-only Docs replays."""

from __future__ import annotations

import sys
from pathlib import Path

from vulkan_docs_observer_model import MAX_RECORDS, OBSERVATION_FIELDS, PHASES, RUN_FIELDS, Observation, ObservationRun, reject
from vulkan_docs_observer_parse import canonical, digest, document, identifier, positive, selector
from vulkan_docs_observer_plan import OUTPUT, SOURCE_TREE, TREE, observer_digest, producer_digest, witness_digest


def run(value: object) -> ObservationRun:
    if not isinstance(value, dict) or set(value) != RUN_FIELDS:
        reject("observer run has an invalid schema")
    identity = identifier(value.get("id"), "observer run")
    artifact = selector(value.get("artifact"), "observer artifact")
    if not artifact.startswith("runs/"):
        reject("observer artifact escapes the declared artifact root")
    if value.get("source_tree_sha256") != SOURCE_TREE[2] or value.get("generated_tree_sha256") != TREE[2]:
        reject("observer run does not bind the reviewed source/output trees")
    if value.get("primary_html_sha256") != OUTPUT["sha256"]:
        reject("observer run does not bind the reviewed primary HTML")
    for field in ("io_trace_sha256", "include_trace_sha256", "input_manifest_sha256", "include_identity_sha256"):
        digest(value.get(field), f"observer run {field}")
    if digest(value.get("producer_argv_sha256"), "observer producer argv") != producer_digest():
        reject("observer run does not retain the exact pinned producer argv")
    raw = positive(value.get("raw_count"), "observer raw count", 8192)
    derived = positive(value.get("derived_count"), "observer derived count", 8192)
    included = positive(value.get("include_count"), "observer include count", 8192)
    if raw + derived > MAX_RECORDS:
        reject("observer run exceeds its bounded input count")
    phases = value.get("phase_counts")
    if not isinstance(phases, dict) or set(phases) != PHASES:
        reject("observer phase counts have an invalid schema")
    if any(positive(phases[name], f"observer phase {name}", 8192) < 1 for name in PHASES):
        reject("observer phase counts are empty")
    if any(phases[name] > raw + derived for name in PHASES):
        reject("observer phase count exceeds captured records")
    actual = digest(value.get("run_sha256"), "observer run sha256")
    expected = dict(value)
    expected.pop("run_sha256", None)
    if actual != canonical(expected, "webboxvm-graphics-vulkan-docs-observation-run-v1"):
        reject("observer run has a stale run_sha256")
    return ObservationRun(identity, actual, raw, derived, included)


def observation_value(value: object) -> Observation:
    if not isinstance(value, dict) or set(value) != OBSERVATION_FIELDS:
        reject("observer header has an invalid schema")
    expected = (1, "vulkan-docs-core-input-observation-v1", "input-observation-only-unadmitted", "vulkan-1.4-core",
                "api-limit-format-spec", "vulkan-14-spec")
    if type(value.get("schema")) is not int or tuple(value.get(field) for field in (
            "schema", "contract", "status", "profile", "role", "required_input_id")) != expected:
        reject("observer header can be mistaken for an admitted closure")
    if digest(value.get("build_witness_sha256"), "observer witness") != witness_digest():
        reject("observer header does not bind the pinned build witness")
    if digest(value.get("observer_source_sha256"), "observer source") != observer_digest():
        reject("observer header does not bind the reviewed observer source")
    rows = value.get("runs")
    if not isinstance(rows, list) or len(rows) != 2:
        reject("observer header needs exactly two independent run records")
    parsed = tuple(run(item) for item in rows)
    if len({item.identifier for item in parsed}) != len(parsed) or len({item.digest for item in parsed}) != len(parsed):
        reject("observer header repeats a run identity")
    if len({item.get("artifact") for item in rows}) != len(rows):
        reject("observer header repeats a raw artifact location")
    actual = digest(value.get("observation_sha256"), "observer header sha256")
    expected_value = dict(value)
    expected_value.pop("observation_sha256", None)
    if actual != canonical(expected_value, "webboxvm-graphics-vulkan-docs-observation-v1"):
        reject("observer header has a stale observation_sha256")
    return Observation(actual, parsed)


def observation(path: Path) -> Observation:
    return observation_value(document(path))


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: vulkan_docs_observer_contract.py OBSERVATION.json")
    try:
        result = observation(Path(sys.argv[1]))
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"OBSERVATION: {len(result.runs)} runs, {result.runs[0].raw_count} raw, {result.runs[0].derived_count} derived, 0 cutover-ready")


if __name__ == "__main__":
    main()
