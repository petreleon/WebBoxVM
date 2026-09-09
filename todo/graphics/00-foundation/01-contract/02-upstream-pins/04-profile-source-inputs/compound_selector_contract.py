#!/usr/bin/env python3
"""Fail-closed exact validation for F02.4 compound selector audits."""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path, PurePosixPath
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
CLOSURE_FIELDS = frozenset(("schema", "profile", "candidate_id", "candidate_sha256", "included", "excluded", "core_case_count"))
CONFIGURATION_FIELDS = frozenset(("schema", "profile", "candidate_id", "candidate_sha256", "configurations",
                                  "excluded_configurations", "core_configuration_count", "core_case_configuration_runs",
                                  "excluded_configuration_count"))
INPUT_FIELDS = ("id", "source_family", "immutable_url", "revision", "sha256", "bytes", "license", "local_cache",
                "generated_code_role", "provenance")
COMPONENT_FIELDS = frozenset((*INPUT_FIELDS, "selector", "case_count", "reason"))
CONFIG_FIELDS = frozenset(("selector", "command_line", "name", "os", "use_for_first_egl_config", "reason"))


class CompoundError(ValueError):
    """A compound selector audit is incomplete or not pinned exactly."""


def reject(message: str) -> None:
    raise CompoundError(message)


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


CANDIDATES = reviewed_module("f024_candidate_contract_for_compound", HERE / "candidate_contract.py")
CATALOG = reviewed_module("f024_compound_catalog", HERE / "compound_catalog.py").CATALOG


def document(path: Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"compound {label} audit cannot be read: {error}")
    if not isinstance(value, dict):
        reject(f"compound {label} audit is not a JSON object")
    return value


def selector(value: object, url: object) -> None:
    if not isinstance(value, str) or not isinstance(url, str) or not value:
        reject("compound selector has an empty selector or URL")
    path = PurePosixPath(value)
    if path.is_absolute() or str(path) != value or ".." in path.parts:
        reject("compound selector path is unsafe")
    if not urlsplit(url).path.endswith("/" + value):
        reject("compound selector does not name its pinned source")


def source_policy(value: dict[str, object]) -> None:
    try:
        CANDIDATES.SourceInput.from_manifest({field: value[field] for field in INPUT_FIELDS})
    except CANDIDATES.ContractError as error:
        reject(f"compound selector violates F02.2 policy: {error}")


def components(value: object, expected: tuple[dict[str, object], ...], label: str) -> dict[str, int]:
    if not isinstance(value, list) or tuple(value) != expected:
        reject(f"compound {label} does not match the reviewed exact closure")
    counts: dict[str, int] = {}
    for item in value:
        if not isinstance(item, dict) or set(item) != COMPONENT_FIELDS:
            reject(f"compound {label} has an invalid component schema")
        if type(item["case_count"]) is not int or item["case_count"] < 1:
            reject(f"compound {label} has an invalid case count")
        selector(item["selector"], item["immutable_url"])
        source_policy(item)
        counts[item["selector"]] = item["case_count"]
    if len(counts) != len(value):
        reject(f"compound {label} repeats a selector")
    return counts


def configurations(value: object, expected: tuple[dict[str, object], ...], counts: dict[str, int], label: str) -> tuple[int, int]:
    if not isinstance(value, list) or tuple(value) != expected:
        reject(f"compound {label} does not match the reviewed exact configurations")
    for item in value:
        strings = ("selector", "command_line", "name", "os", "reason")
        if (not isinstance(item, dict) or set(item) != CONFIG_FIELDS or item["selector"] not in counts
                or any(not isinstance(item[field], str) or not item[field] for field in strings)
                or type(item["use_for_first_egl_config"]) is not bool):
            reject(f"compound {label} has an invalid configuration")
    return len(value), sum(counts[item["selector"]] for item in value)


def rejected_candidate(profile: str, candidates: Path, expected: dict[str, object]) -> None:
    identity = CANDIDATES.CATALOG.get(profile, {}).get(expected["candidate_id"])
    if not isinstance(identity, dict) or expected["candidate_sha256"] != identity.get("sha256"):
        reject("compound catalog does not bind the reviewed descriptor digest")
    try:
        requirements = CANDIDATES.requirements(profile)
        decisions = CANDIDATES.validate(candidates, profile)
    except CANDIDATES.AuditError as error:
        reject(f"compound audit candidate contract failed: {error}")
    matching = [decision for (identifier, role), decision in zip(requirements, decisions)
                if identifier == expected["candidate_id"] and role == "conformance-manifest"]
    if matching != ["rejected"]:
        reject("compound audit does not bind a rejected conformance candidate")


def whole_count(value: dict[str, object], name: str) -> int:
    number = value.get(name)
    if type(number) is not int or number < 0:
        reject(f"compound selector audit has an invalid {name}")
    return number


def header(value: dict[str, object], fields: frozenset[str], profile: str, expected: dict[str, object], label: str) -> None:
    if set(value) != fields or type(value.get("schema")) is not int or value["schema"] != 1 or value.get("profile") != profile:
        reject(f"compound {label} audit does not match schema version 1")
    if value.get("candidate_id") != expected["candidate_id"] or value.get("candidate_sha256") != expected["candidate_sha256"]:
        reject(f"compound {label} audit does not bind the reviewed rejected candidate")


def validate(path: Path, configurations_path: Path, candidates: Path, profile: str) -> tuple[int, int, int, int, int, int]:
    expected = CATALOG.get(profile)
    if not isinstance(expected, dict):
        reject("compound selector audit has an unknown target profile")
    rejected_candidate(profile, candidates, expected)
    closure, configurations_doc = document(path, "closure"), document(configurations_path, "configuration")
    header(closure, CLOSURE_FIELDS, profile, expected, "closure")
    header(configurations_doc, CONFIGURATION_FIELDS, profile, expected, "configuration")
    included = components(closure.get("included"), expected["included"], "included closure")
    excluded = components(closure.get("excluded"), expected["excluded"], "excluded boundary")
    core_configs, core_runs = configurations(configurations_doc.get("configurations"), expected["configurations"], included, "core configurations")
    excluded_configs, _ = configurations(configurations_doc.get("excluded_configurations"), expected["excluded_configurations"], excluded, "excluded configurations")
    if (whole_count(closure, "core_case_count") != sum(included.values())
            or whole_count(configurations_doc, "core_configuration_count") != core_configs
            or whole_count(configurations_doc, "core_case_configuration_runs") != core_runs
            or whole_count(configurations_doc, "excluded_configuration_count") != excluded_configs):
        reject("compound selector audit has an incorrect aggregate count")
    return len(included), sum(included.values()), core_configs, core_runs, len(excluded), excluded_configs


def main() -> None:
    if len(sys.argv) != 5:
        raise SystemExit("usage: compound_selector_contract.py CLOSURE.json CONFIGURATIONS.json CANDIDATES.json PROFILE")
    try:
        selectors, cases, configs, runs, excluded, excluded_configs = validate(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]), sys.argv[4])
    except CompoundError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"COMPOUND: {sys.argv[4]} {selectors} core selectors, {cases} unique cases, {configs} configurations, {runs} case-configuration runs, {excluded} excluded selector, {excluded_configs} excluded configuration")


if __name__ == "__main__":
    main()
