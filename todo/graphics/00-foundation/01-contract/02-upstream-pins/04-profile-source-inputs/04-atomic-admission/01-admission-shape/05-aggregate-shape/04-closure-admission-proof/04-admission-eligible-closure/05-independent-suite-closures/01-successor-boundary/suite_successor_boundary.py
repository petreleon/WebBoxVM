#!/usr/bin/env python3
"""Freeze one read-only, non-admitting boundary for independent suite successors."""

import hashlib, importlib.util, json, os, stat, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
PROFILE_INPUTS = HERE.parents[6]
AUDIT = PROFILE_INPUTS / "02-gles-input-audit"
CANDIDATES, CLOSURE, CONFIGURATIONS = (AUDIT / "candidates.json", AUDIT / "cts_closure.json",
                                        AUDIT / "cts_configurations.json")
PROOF = HERE.parents[2]
RECORD = HERE / "suite_successor_boundary.json"
V2_HANDOFF = HERE.parents[3] / "05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff.py"
RECONCILER = PROOF / "02-reconcile-current-evidence/current_evidence_reconciler.py"
REQUIREMENTS = HERE.parents[8] / "03-feature-matrix/01-profile-scope/source_requirements.json"
INVENTORY_LOCK = HERE.parents[7] / "01-input-inventory/inventory.lock"
MAX_BYTES = 64 * 1024
EFFECTS = ("active_inventory_changed", "cache_freshness_proved", "f03_changed", "admission_eligible",
           "admitted", "cutover_ready", "supported", "conformant", "certified", "near_native",
           "performance_claimed", "satisfies_vulkan_14_core_manifest")


class BoundaryError(ValueError):
    """A policy boundary is malformed, stale, or falsely promoting a suite."""


def reject(message: str) -> None:
    raise BoundaryError(message)

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

COMPOUND = reviewed("f025_boundary_compound", PROFILE_INPUTS / "compound_selector_contract.py")
SHAPE = reviewed("f025_boundary_shape", HERE.parents[4] / "closure_shape_contract.py")
HANDOFF = reviewed("f025_boundary_handoff", V2_HANDOFF)
RECON = reviewed("f025_boundary_reconciler", RECONCILER)


def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value:
            reject("policy has a duplicate JSON key")
        value[key] = item
    return value

def source(path: Path, label: str, limit: int = MAX_BYTES) -> bytes:
    try:
        if not stat.S_ISREG(path.lstat().st_mode):
            reject(f"{label} is not a regular file")
        descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(descriptor, "rb") as opened:
            if not stat.S_ISREG(os.fstat(descriptor).st_mode):
                reject(f"{label} is not a regular file")
            raw = opened.read(limit + 1)
    except OSError as error:
        reject(f"{label} cannot be read: {error}")
    if len(raw) > limit:
        reject(f"{label} exceeds its bounded size")
    return raw

def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(source(path, "policy").decode("utf-8"), object_pairs_hook=pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"policy cannot be parsed: {error}")
    if not isinstance(value, dict):
        reject("policy is not an object")
    return value

def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "policy_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()

def alike(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right


def hashes() -> dict[str, str]:
    paths = {
        "candidate_audit_document_sha256": CANDIDATES, "gles_closure_document_sha256": CLOSURE,
        "gles_configurations_document_sha256": CONFIGURATIONS, "vcts_handoff_document_sha256": HANDOFF.HANDOFF,
        "vcts_taxonomy_document_sha256": HANDOFF.TAXONOMY_RECEIPT,
        "blocked_receipt_document_sha256": PROOF / "03-blocked-state-receipt/blocked_state_receipt.json",
        "reconciler_document_sha256": RECONCILER, "source_requirements_document_sha256": REQUIREMENTS,
        "inventory_lock_document_sha256": INVENTORY_LOCK,
    }
    return {key: hashlib.sha256(source(path, key)).hexdigest() for key, path in paths.items()}


def current() -> tuple[dict[str, object], dict[str, object]]:
    try:
        compound = COMPOUND.validate(CLOSURE, CONFIGURATIONS, CANDIDATES, "gles-3.2")
        shape = SHAPE.validate(CLOSURE, CONFIGURATIONS, CANDIDATES, "gles-3.2")
        handoff = HANDOFF.validate()
        root, _, _, _, view = HANDOFF.inputs(HANDOFF.IDENTITY, HANDOFF.PLAN, HANDOFF.LEDGER,
                                              HANDOFF.CACHE_RECEIPT, HANDOFF.CAPTURE_RECEIPT,
                                              HANDOFF.TAXONOMY_RECEIPT)
        reconciled = RECON.validate()
        ledger = json.loads(source(HANDOFF.LEDGER, "VCTS ledger", 256 * 1024).decode("utf-8"))["members"]
    except Exception as error:
        reject(f"native suite evidence is invalid: {error}")
    categories = {name: sum(row["category"] == name for row in view.members)
                  for name in ("core", "wsi", "video", "extension", "unknown")}
    if (compound != (4, 12477, 12, 30574, 1, 1) or shape.state != "unadmitted"
            or shape.required_input_id != "gles-cts-manifest" or tuple(shape.core_member_ids) != tuple(
                row["id"] for row in COMPOUND.CATALOG["gles-3.2"]["included"])
            or reconciled.first_blocker != ("gles-cts-manifest", "requires-multifile-core-selector-closure")
            or handoff["member_count"] != 98 or handoff["member_total_bytes"] != 434669348
            or categories != {"core": 0, "wsi": 1, "video": 1, "extension": 4, "unknown": 92}
            or (handoff["admitted"], handoff["cutover_ready"], handoff["satisfies_vulkan_14_core_manifest"]) != (False, False, False)):
        reject("native suite evidence no longer has the reviewed blocked state")
    gles = {"candidate_id": shape.required_input_id, "candidate_sha256": shape.root_sha256,
            "decision": "rejected", "blocker": reconciled.first_blocker[1],
            "core_members": [[row[key] for key in ("id", "selector", "sha256", "bytes")]
                             for row in COMPOUND.CATALOG["gles-3.2"]["included"]],
            "core_case_count": compound[1], "core_configuration_count": compound[2],
            "core_case_configuration_runs": compound[3],
            "excluded_extension": [COMPOUND.CATALOG["gles-3.2"]["excluded"][0][key]
                                   for key in ("id", "selector", "sha256", "bytes")],
            "excluded_configuration_count": compound[5], "successor_closure_ready": False,
            "future_requirements": ["complete-immutable-successor-closure", "per-member-f02-2-validation",
                                    "fresh-authorized-capture-and-offline-replay"]}
    vcts = {"canonical_suite_id": root.suite_id, "root_identity_sha256": root.digest,
            "handoff_sha256": handoff["handoff_sha256"], "taxonomy_sha256": handoff["taxonomy_sha256"],
            "member_count": handoff["member_count"], "member_total_bytes": handoff["member_total_bytes"],
            "category_counts": categories, "selector_scope": handoff["selector_scope"],
            "local_filtering": handoff["local_filtering"], "f02_source_member_cap_bytes": 8 * 1024 * 1024,
            "v2_diagnostic_member_cap_bytes": 64 * 1024 * 1024,
            "over_f02_cap_member_count": sum(row["bytes"] > 8 * 1024 * 1024 for row in ledger),
            "local_selector_allowed": False, "taxonomy_is_conformance": False, "core_manifest_ready": False,
            "future_requirements": ["khronos-published-immutable-vulkan-1.4-core-manifest",
                                    "complete-recursive-f02-2-member-validation",
                                    "no-local-selector-or-taxonomy-substitution"]}
    return gles, vcts

def validate(path: Path = RECORD) -> dict[str, object]:
    value = document(path)
    fields = {"schema", "kind", "status", "anchors", "gles", "vcts", "effects", "policy_sha256"}
    if (set(value) != fields or type(value.get("schema")) is not int
            or (value["schema"], value["kind"], value["status"]) != (1, "independent-suite-successor-policy-v1", "policy-only-unadmitted")):
        reject("policy has an unexpected schema or status")
    if not isinstance(value.get("policy_sha256"), str) or value["policy_sha256"] != digest(value):
        reject("policy self-hash is invalid")
    gles, vcts = current()
    if (not alike(value["anchors"], hashes()) or not alike(value["gles"], gles) or not alike(value["vcts"], vcts)
            or not alike(value["effects"], {name: False for name in EFFECTS})):
        reject("policy does not bind the exact blocked successor boundary")
    return value

def main() -> None:
    try:
        value = validate(Path(sys.argv[1]) if len(sys.argv) == 2 else RECORD) if len(sys.argv) < 3 else None
        if value is None:
            raise SystemExit("usage: suite_successor_boundary.py [POLICY.json]")
    except BoundaryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"POLICY: {value['status']} {value['policy_sha256']}")


if __name__ == "__main__":
    main()
