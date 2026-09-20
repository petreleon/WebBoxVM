"""Validate a no-claim, source-sealed F05 profile-registration catalog."""

from __future__ import annotations

import copy
import hashlib
import re
from pathlib import Path

import profile_registration_sources as sources
from profile_registration_common import (
    RegistrationError, argv, canonical, document, no_claims, prerequisites, reject, relative, text,
)

HERE = Path(__file__).resolve().parent
CATALOG = HERE / "profile_registration_catalog.json"
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]*$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
CLAIMS = frozenset(("khronos_selector", "api_support", "conformance", "certification", "profile_support", "performance"))
ROOT_FIELDS = frozenset(("schema", "kind", "source_contract_sha256", "inventory_lock_sha256", "claims", "cts_executions", "states", "checks", "catalog_sha256"))
STATE_FIELDS = frozenset(("profile_implementation_count", "profile_status", "blocker", "api_support", "conformance", "certification", "profile_support", "performance", "guest_api", "browser_execution"))
CHECK_FIELDS = frozenset(("id", "kind", "profile", "profile_effect", "source", "command", "expected_count", "artifacts", "tools", "prerequisites", "evidence", "lane", "requires_selector_cache"))
KINDS = frozenset(("implementation", "auxiliary-inventory", "baseline"))
LANES = frozenset(("make-test", "wasm-serial", "wasm-threaded", "transport-only", "auxiliary-inventory", "implementation"))


def source_maps(contract: dict[str, object]) -> tuple[dict[str, dict[str, object]], dict[str, dict[str, object]], dict[tuple[str, str], dict[str, object]]]:
    records = {item.get("id"): item for item in contract.get("records", []) if isinstance(item, dict)}
    auxiliary = {item.get("id"): item for item in contract.get("auxiliary", []) if isinstance(item, dict)}
    bindings = {(item.get("profile"), item.get("role")): item for item in contract.get("bindings", []) if isinstance(item, dict)}
    if len(records) != 8 or len(auxiliary) != 4 or len(bindings) != 6 or None in records or None in auxiliary or None in bindings:
        reject("sealed F02 descriptor is incomplete")
    return records, auxiliary, bindings


def validate_source(check: dict[str, object], records: dict[str, dict[str, object]], auxiliary: dict[str, dict[str, object]], bindings: dict[tuple[str, str], dict[str, object]]) -> None:
    kind, profile, source = check["kind"], check["profile"], check["source"]
    if kind == "baseline":
        if profile is not None or source is not None or check["profile_effect"] != "none" or check["requires_selector_cache"]:
            reject("baseline lane cannot become a profile claim")
        return
    if not isinstance(profile, str) or (profile, "normative-root") not in bindings:
        reject("profile does not resolve to an admitted source pair")
    if kind == "implementation":
        expected = [bindings[(profile, "normative-root")], bindings[(profile, "full-suite-root")]]
        if check["profile_effect"] != "blocked-observation" or not isinstance(source, dict) or source.get("roles") != expected:
            reject("implementation check has a stale or cross-profile source-role pair")
        return
    if check["profile_effect"] != "none" or check["requires_selector_cache"] is not True or not isinstance(source, dict):
        reject("auxiliary inventory has an unsafe profile effect")
    identifier = source.get("id")
    record, evidence = records.get(identifier), auxiliary.get(identifier)
    fields = ("id", "record_kind", "scope", "revision", "sha256", "bytes")
    if (identifier != "vulkan-registry" or not isinstance(record, dict) or not isinstance(evidence, dict)
            or set(source) != set(fields) or source.get("record_kind") != record.get("kind")
            or any(source[key] != record.get(key) for key in ("id", "revision", "sha256", "bytes"))
            or any(evidence.get(key) != source[key] for key in ("id", "record_kind", "scope", "sha256"))
            or not no_claims(evidence.get("claims"), CLAIMS) or evidence.get("cts_executions") != 0
            or evidence.get("discharges_required_role") is not False or any(item.get("record_id") == identifier for item in bindings.values())):
        reject("auxiliary inventory attempts to substitute a mandatory source role")


def validate_check(check: object, records: dict[str, dict[str, object]], auxiliary: dict[str, dict[str, object]], bindings: dict[tuple[str, str], dict[str, object]]) -> dict[str, object]:
    if not isinstance(check, dict) or set(check) != CHECK_FIELDS or check.get("kind") not in KINDS:
        reject("registration check has an unexpected schema")
    if not IDENTIFIER.fullmatch(text(check["id"], "check id")) or check.get("lane") not in LANES:
        reject("registration check has an invalid id or lane")
    if type(check["expected_count"]) is not int or check["expected_count"] <= 0:
        reject("registration check needs a positive expected count")
    argv(check["command"], "command")
    if not isinstance(check["tools"], list) or not check["tools"]:
        reject("registration check needs at least one tool")
    for tool in check["tools"]:
        argv(tool, "tool")
    if not isinstance(check["artifacts"], list) or not check["artifacts"]:
        reject("registration check needs a nonempty artifact list")
    for item in check["artifacts"]:
        relative(item, "artifact", exists=True)
    if not isinstance(check["evidence"], str):
        reject("evidence must be text")
    relative(check["evidence"].partition("#")[0], "evidence", exists=True)
    prerequisites(check["prerequisites"])
    validate_source(check, records, auxiliary, bindings)
    return copy.deepcopy(check)


def validate_catalog(value: object, contract: dict[str, object] | None = None) -> dict[str, object]:
    contract = sources.admitted_contract() if contract is None else contract
    contract_states = contract.get("states") if isinstance(contract, dict) else None
    if (not isinstance(contract_states, dict) or contract_states.get("source_contract_complete") is not True
            or contract_states.get("f05_profile_registration") is not False
            or not no_claims(contract.get("claims"), CLAIMS) or contract.get("cts_executions") != 0):
        reject("sealed F02 descriptor cannot admit profile registration")
    if not isinstance(value, dict) or set(value) != ROOT_FIELDS or value.get("schema") != 1 or value.get("kind") != "webboxvm-f05-profile-registration":
        reject("registration catalog has an unexpected schema")
    if (not SHA256.fullmatch(str(value.get("source_contract_sha256", "")))
            or not SHA256.fullmatch(str(value.get("inventory_lock_sha256", "")))
            or value["source_contract_sha256"] != contract.get("source_contract_sha256")
            or value["inventory_lock_sha256"] != contract.get("inventory_lock_sha256")
            or not no_claims(value.get("claims"), CLAIMS) or value.get("cts_executions") != 0):
        reject("registration catalog has stale or qualifying source state")
    states = value.get("states")
    non_claims = STATE_FIELDS - {"profile_implementation_count", "profile_status", "blocker"}
    if (not isinstance(states, dict) or set(states) != STATE_FIELDS or states.get("profile_status") != "blocked"
            or states.get("blocker") != "matrix-incomplete" or any(states.get(key) is not False for key in non_claims)):
        reject("registration catalog promotes profile state")
    body = {key: item for key, item in value.items() if key != "catalog_sha256"}
    if value.get("catalog_sha256") != hashlib.sha256(canonical(body)).hexdigest():
        reject("registration catalog has a stale self hash")
    records, auxiliary, bindings = source_maps(contract)
    checks = value.get("checks")
    if not isinstance(checks, list) or not checks:
        reject("registration catalog has no checks")
    checked = [validate_check(item, records, auxiliary, bindings) for item in checks]
    if (len({item["id"] for item in checked}) != len(checked)
            or states["profile_implementation_count"] != sum(item["kind"] == "implementation" for item in checked)):
        reject("registration catalog has duplicate ids or a wrong implementation count")
    return copy.deepcopy(value)


def load_catalog(path: Path = CATALOG) -> dict[str, object]:
    return validate_catalog(document(path))
