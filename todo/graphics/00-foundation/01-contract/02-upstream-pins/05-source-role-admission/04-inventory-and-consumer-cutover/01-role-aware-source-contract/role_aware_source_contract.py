#!/usr/bin/env python3
"""Seal the exact role-aware F02.5 source catalog without qualifying a profile."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE_ROLE = HERE.parents[1]
ROLE = SOURCE_ROLE / "01-authority-and-transform-boundary"
NORMATIVE = SOURCE_ROLE / "02-normative-build-record/01-normative-root-pins"
LOCAL = SOURCE_ROLE / "02-normative-build-record/02-vulkan-local-definition"
SUITES = SOURCE_ROLE / "03-full-suite-record/01-canonical-full-suite-roots"
sys.path[:0] = [str(ROLE), str(NORMATIVE), str(LOCAL), str(SUITES), str(HERE)]
import full_suite_roots as suites  # noqa: E402
import normative_roots as normative  # noqa: E402
import role_aware_source_evidence as evidence  # noqa: E402
import vulkan_definition_contract as definition  # noqa: E402
from source_role_contract import validate_catalog  # noqa: E402
from source_role_records import RoleError  # noqa: E402

NO_CLAIMS = dict(evidence.NO_CLAIMS)
PAIRS = (("opengl-4.6-core", "opengl-46-core-spec", "opengl-cts-gl46-main"),
         ("gles-3.2", "gles-32-spec", "gles-cts-main"),
         ("vulkan-1.4-core", "vulkan-14-spec", "vulkan-cts-default"))


class SourceContractError(ValueError):
    """A source contract is stale, mixed, incomplete, or overclaims qualification."""


def reject(message: str) -> None:
    raise SourceContractError(message)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def record_map(records: list[dict[str, object]]) -> dict[str, dict[str, object]]:
    return {str(record["id"]): record for record in records}


def catalog() -> dict[str, object]:
    roots = {item["id"]: copy.deepcopy(item) for item in (*normative.ROOTS, *suites.ROOTS)}
    ordered = [roots["opengl-46-core-spec"], roots["opengl-cts-gl46-main"],
               roots["gles-32-spec"], roots["gles-cts-main"], roots["vulkan-14-spec"],
               roots["vulkan-cts-default"], roots["vulkan-registry"], definition.transform()]
    return {"schema": 1, "records": ordered}


def binding(profile: str, role: str, record: dict[str, object]) -> dict[str, object]:
    value = {"profile": profile, "role": role, "record_id": record["id"],
             "record_kind": record["kind"], "scope": record["scope"],
             "revision": record["revision"], "sha256": record["sha256"], "bytes": record["bytes"]}
    if role == "full-suite-root":
        value.update(suite_id=record["suite_id"], selector_path=record["selector_path"],
                     unfiltered=record["unfiltered"], selector_scope="Khronos full root; not core-only")
    return value


def bindings(records: dict[str, dict[str, object]]) -> list[dict[str, object]]:
    result = []
    for profile, normative_id, suite_id in PAIRS:
        result.extend((binding(profile, "normative-root", records[normative_id]),
                       binding(profile, "full-suite-root", records[suite_id])))
    return result


def body() -> dict[str, object]:
    source_catalog = catalog()
    records = record_map(source_catalog["records"])
    return {"schema": 1, "kind": "webboxvm-role-aware-source-contract",
            "authority": "WebBoxVM", "producer": "WebBoxVM", "claims": dict(NO_CLAIMS),
            "cts_executions": 0, "record_count": len(records), "records": source_catalog["records"],
            "inventory_lock_sha256": hashlib.sha256(canonical(source_catalog)).hexdigest(),
            "bindings": bindings(records), "closures": evidence.closures(records),
            "auxiliary": evidence.auxiliary(records),
            "states": {"mandatory_role_inventory_complete": True, "profile_status": "blocked",
                       "blocker": "matrix-incomplete", "api_support": False, "conformance": False,
                       "certification": False, "profile_support": False, "performance": False}}


def contract() -> dict[str, object]:
    value = body()
    return {**value, "source_contract_sha256": hashlib.sha256(canonical(value)).hexdigest()}


def seal_body() -> dict[str, object]:
    value = contract()
    return {"schema": 1, "kind": "webboxvm-role-aware-source-contract-seal",
            "authority": "WebBoxVM", "producer": "WebBoxVM", "claims": dict(NO_CLAIMS),
            "cts_executions": 0, "source_contract_sha256": value["source_contract_sha256"],
            "inventory_lock_sha256": value["inventory_lock_sha256"],
            "records": [{"id": item["id"], "kind": item["kind"], "revision": item.get("revision"),
                         "sha256": item["sha256"], "bytes": item["bytes"]} for item in value["records"]],
            "bindings": [{key: item[key] for key in ("profile", "role", "record_id")} for item in value["bindings"]],
            "closures": [{key: item[key] for key in ("profile", "root_id", "ledger_sha256", "receipt_sha256")}
                         for item in value["closures"]],
            "auxiliary": [{"id": item["id"], "sha256": item.get("sha256"),
                           "receipt_sha256": item.get("receipt_sha256")} for item in value["auxiliary"]],
            "states": value["states"]}


def seal() -> dict[str, object]:
    value = seal_body()
    return {**value, "seal_sha256": hashlib.sha256(canonical(value)).hexdigest()}


def producer_catalogs() -> None:
    try:
        normative.validate_normative_catalog(normative.catalog())
        suites.validate_full_suite_catalog(suites.catalog())
        definition.checked_record(definition.build_record())
    except (definition.DefinitionError, normative.RootError, suites.FullSuiteError, RoleError) as error:
        reject(str(error))


def validate_contract(value: object) -> tuple[str, ...]:
    if not isinstance(value, dict) or "source_contract_sha256" not in value:
        reject("source contract has an invalid schema")
    body_value = {key: item for key, item in value.items() if key != "source_contract_sha256"}
    if value["source_contract_sha256"] != hashlib.sha256(canonical(body_value)).hexdigest():
        reject("source contract has a stale self hash")
    producer_catalogs()
    expected = contract()
    if value != expected:
        reject("source contract differs from the exact reviewed role catalog")
    try:
        identifiers = validate_catalog({"schema": 1, "records": value["records"]})
    except RoleError as error:
        reject(str(error))
    evidence.validate_receipt_files(value["closures"], value["auxiliary"])
    return identifiers


def validate_seal(value: object) -> None:
    if not isinstance(value, dict) or "seal_sha256" not in value:
        reject("source contract seal has an invalid schema")
    body_value = {key: item for key, item in value.items() if key != "seal_sha256"}
    if value["seal_sha256"] != hashlib.sha256(canonical(body_value)).hexdigest():
        reject("source contract seal has a stale self hash")
    if value != seal():
        reject("source contract seal differs from the exact reviewed lock")
    validate_contract(contract())


def binding_for(value: object, profile: str, role: str) -> dict[str, object]:
    validate_contract(value)
    matches = [item for item in value["bindings"] if item["profile"] == profile and item["role"] == role]
    if len(matches) != 1:
        reject("profile role does not resolve exactly once")
    return copy.deepcopy(matches[0])


def main() -> None:
    if sys.argv[1:]:
        raise SystemExit("usage: role_aware_source_contract.py")
    try:
        print(json.dumps(contract(), sort_keys=True))
    except (SourceContractError, evidence.EvidenceError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
