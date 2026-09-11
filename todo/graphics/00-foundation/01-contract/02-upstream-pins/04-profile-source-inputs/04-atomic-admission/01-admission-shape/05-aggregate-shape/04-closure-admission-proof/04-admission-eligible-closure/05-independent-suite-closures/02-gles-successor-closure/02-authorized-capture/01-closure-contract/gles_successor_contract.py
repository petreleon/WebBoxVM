#!/usr/bin/env python3
"""Freeze a hostile-tested, successor-only contract for the Khronos GLES closure."""

import hashlib, importlib.util, json, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
INTEGRATION_DIR = HERE.parents[1] / "01-successor-integration"
RECORD = HERE / "gles_successor_contract.json"
EFFECTS = ("active_inventory_changed", "cache_freshness_proved", "f03_changed", "admission_eligible",
           "admitted", "cutover_ready", "supported", "conformant", "certified", "near_native",
           "performance_claimed", "satisfies_vulkan_14_core_manifest")


class ContractError(ValueError):
    """The successor contract is malformed, stale, or falsely admitting."""


def reject(message: str) -> None:
    raise ContractError(message)


def reviewed(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed module {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(spec.name, None)
        raise
    return module


INTEGRATION = reviewed("f025_gles_successor_integration", INTEGRATION_DIR / "multi_suite_successor_integration.py")
BOUNDARY, COMPOUND = INTEGRATION.BOUNDARY, INTEGRATION.BOUNDARY.COMPOUND


def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value:
            reject("contract has a duplicate JSON key")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        raw = INTEGRATION.raw(path, "contract")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except (INTEGRATION.IntegrationError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"contract cannot be read: {error}")
    if not isinstance(value, dict):
        reject("contract is not an object")
    return value


def sha(path: Path) -> str:
    try:
        return hashlib.sha256(INTEGRATION.raw(path, "contract anchor")).hexdigest()
    except INTEGRATION.IntegrationError as error:
        reject(f"contract anchor cannot be read: {error}")


def digest(value: dict[str, object], omitted: str = "contract_sha256") -> str:
    body = {key: item for key, item in value.items() if key != omitted}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def alike(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right


def member(row: dict[str, object], cases: int) -> dict[str, object]:
    return {key: row[key] for key in ("id", "selector", "sha256", "bytes")} | {"case_count": cases}


def anchors() -> dict[str, str]:
    return {"integration_document_sha256": sha(INTEGRATION.RECORD),
            "candidate_audit_document_sha256": sha(BOUNDARY.CANDIDATES),
            "closure_document_sha256": sha(BOUNDARY.CLOSURE),
            "configurations_document_sha256": sha(BOUNDARY.CONFIGURATIONS)}


def expected() -> dict[str, object]:
    try:
        integration = INTEGRATION.validate()
        counts = COMPOUND.validate(BOUNDARY.CLOSURE, BOUNDARY.CONFIGURATIONS, BOUNDARY.CANDIDATES, "gles-3.2")
        catalog = COMPOUND.CATALOG["gles-3.2"]
        root = {"id": "gles-cts-manifest", **COMPOUND.CANDIDATES.CATALOG["gles-3.2"]["gles-cts-manifest"]}
    except Exception as error:
        reject(f"reviewed GLES predecessor is invalid: {error}")
    core, excluded, configurations = catalog["included"], catalog["excluded"], catalog["configurations"]
    if (counts, len(core), len(excluded), len(configurations), integration["gles"]["successor_closure_ready"]) != (
            (4, 12477, 12, 30574, 1, 1), 4, 1, 12, False):
        reject("reviewed GLES closure has drifted")
    revision, raw_prefix = root["revision"], root["immutable_url"].removesuffix(root["selector"])
    source = {"revision": revision, "raw_url_prefix": raw_prefix, "provenance": root["provenance"],
              "license": root["license"], "root_source_family": root["source_family"],
              "member_source_family": core[0]["source_family"], "member_max_bytes": 8 * 1024 * 1024,
              "cache_template": "webboxvm-graphics/f02/{id}/{sha256}.source",
              "active_f02_cache_grammar_compatible": True}
    rows = [*core, *excluded]
    if (root["immutable_url"] != raw_prefix + root["selector"] or root["bytes"] > source["member_max_bytes"]
            or any(row["revision"] != revision or row["immutable_url"] != raw_prefix + row["selector"]
           or row["license"] != source["license"] or row["provenance"] != source["provenance"]
           or row["source_family"] != source["member_source_family"]
           or row["bytes"] > source["member_max_bytes"] for row in rows)):
        reject("reviewed GLES members do not share the pinned source policy")
    if any(row["local_cache"] != source["cache_template"].format(id=row["id"], sha256=row["sha256"])
           for row in [root, *rows]):
        reject("reviewed GLES members do not use the F02 cache grammar")
    values = anchors()
    anchor = values
    root_value, core_values, excluded_value = member(root, counts[1]), [member(row, row["case_count"]) for row in core], member(excluded[0], excluded[0]["case_count"])
    closure_seed = {"profile": "gles-3.2", "source": source, "root": root_value, "core_members": core_values,
                    "excluded_extension": excluded_value, "configurations_document_sha256": anchor["configurations_document_sha256"]}
    closure_sha256 = digest(closure_seed, "")
    cache = {row["id"]: source["cache_template"].format(id=row["id"], sha256=row["sha256"])
             for row in [root_value, *core_values, excluded_value]}
    closure = {"profile": "gles-3.2", "family": "gles-cts", "root": root_value, "core_members": core_values,
               "excluded_extension": excluded_value, "configuration_document_sha256": anchor["configurations_document_sha256"],
               "core_configuration_count": counts[2], "core_case_configuration_runs": counts[3],
               "excluded_configuration_count": counts[5], "root_and_nonroot_required": True,
               "optional_extension_excluded": True, "closure_sha256": closure_sha256, "successor_cache_paths": cache}
    predecessor = {"integration_sha256": integration["integration_sha256"], "active_schema": integration["predecessor"]["active_schema"],
                   "inventory_lock_sha256": integration["predecessor"]["inventory_lock_sha256"],
                   "active_family_alias_allowed": False, "active_mutation_permitted": False}
    producer = {"repository_revision": revision, "entrypoints": [
                    {"selector": "external/openglcts/scripts/build_mustpass.py", "role": "mustpass-project-composition"},
                    {"selector": "external/openglcts/scripts/mustpass.py", "role": "mustpass-list-and-manifest-emission"}],
                "entrypoints_are_captured_members": False,
                "producer_execution_required_for_this_capture": False,
                "producer_execution_proved": False, "output_attestation_present": False}
    capture = {"fresh_complete_capture_required": True, "offline_replay_required": True,
               "network_during_replay_allowed": False, "atomic_f02_f03_revalidation_required": True,
               "active_cache_freshness_proved": False}
    return {"anchors": values, "predecessor": predecessor, "source": source, "closure": closure,
            "producer": producer, "capture": capture}


def validate(path: Path = RECORD) -> dict[str, object]:
    value = document(path)
    fields = {"schema", "contract", "status", "anchors", "predecessor", "source", "closure", "producer", "capture", "effects", "contract_sha256"}
    if (set(value) != fields or type(value.get("schema")) is not int
            or (value["schema"], value["contract"], value["status"]) != (1, "gles-successor-closure-contract-v1", "design-only-unadmitted")):
        reject("contract has an unexpected schema or status")
    if not isinstance(value.get("contract_sha256"), str) or value["contract_sha256"] != digest(value):
        reject("contract self-hash is invalid")
    if (not alike({key: value[key] for key in ("anchors", "predecessor", "source", "closure", "producer", "capture")}, expected())
            or not alike(value["effects"], {name: False for name in EFFECTS})):
        reject("contract does not bind the exact unadmitted GLES closure")
    return value


def main() -> None:
    try:
        value = validate(Path(sys.argv[1]) if len(sys.argv) == 2 else RECORD) if len(sys.argv) < 3 else None
        if value is None:
            raise SystemExit("usage: gles_successor_contract.py [CONTRACT.json]")
    except ContractError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"CONTRACT: {value['status']} {value['contract_sha256']}")


if __name__ == "__main__":
    main()
