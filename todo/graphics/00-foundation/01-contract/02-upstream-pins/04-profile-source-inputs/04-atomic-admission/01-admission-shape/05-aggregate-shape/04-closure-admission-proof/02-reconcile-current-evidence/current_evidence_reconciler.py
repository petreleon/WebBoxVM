#!/usr/bin/env python3
"""Reconcile the current V1 and V2 evidence into one read-only blocked result."""

from __future__ import annotations

import hashlib, importlib.util, sys
from dataclasses import dataclass, field
from pathlib import Path

HERE = Path(__file__).resolve().parent
SHAPE = HERE.parent.parent
def reviewed(name: str, path: Path):
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
PROOF = reviewed("f024_admission_proof_contract", HERE.parent / "01-proof-contract/admission_proof_contract.py")
PRE = reviewed("f024_current_pre_admission", SHAPE / "01-pre-admission-aggregate/aggregate_contract.py")
V2 = reviewed("f024_current_v2_handoff", SHAPE / "05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff.py")
V1_PATHS = (PRE.TRANSITION.RULES, PRE.TRANSITION.MAP.SOURCE_MAP, PRE.TRANSITION.BOUNDARY.BOUNDARIES)
V2_PATHS = (V2.HANDOFF, V2.IDENTITY, V2.PLAN, V2.LEDGER, V2.CACHE_RECEIPT, V2.CAPTURE_RECEIPT, V2.TAXONOMY_RECEIPT)
V1_PROFILES = tuple(PRE.TRANSITION.MAP.AUDITS)
V1_NAMES = ("rules", "source-map", "boundaries", *(f"audit:{profile}" for profile in V1_PROFILES))
V1_LOCK = ("74300113dab0019c25b1ed629d13c7fbba720316f370a7deee1cdb86b56d91bc",
           "7f3ee1a9e4faeb201de3775517f64d3f32f8b0805fd45b4efcfbea017b770785",
           "a676b29421b2681e074264175dbae758dfbdb56e7edfd48b5858ba17513a1a90",
           "2ee3465335928146c016e570d9bed6b9b9369aed12e6847b2726149a2610d2a5",
           "3d1d6d523026bdeb5e16c967750237f64a6fc134893348620c0bc4c58aff1a52",
           "7aa23749e5af061221d3f2aced93f3f6e2ebfd3d37445ace0834cd7440ec39c3")
V2_DIGESTS = ("handoff_sha256", "root_identity_sha256", "tree_plan_sha256", "ledger_sha256",
              "cache_receipt_sha256", "capture_receipt_sha256", "taxonomy_sha256")
V2_LOCK = ("8b286471e95a957c7d1c9fc9cf41c26d246de86ad1a7cbffb4fb58c3fde682ee",
           "30b272f8c563e0dabf307795c01496eb70f744514b4790439ebbfc69c7bd5218",
           "77875b165876a3cf37450667c7e9c23e177bff8a7795e0d48b4214e202064804",
           "608d520463bfd4724e79b52ee639a12d446e14c5e8d7b3687872eaf3c4e917f0",
           "56fd45c16f1f1e1c68bfc76f3853215eaff08c71ec60e5ceb85c5ecb26ef8437",
           "48c4d54d8738a8eab526347bc8080c245bb9cef269b66011e32116f78f481599",
           "752a3ae5ff10ea0d9f9a2638c0d1612fb7af3d376dabad1601f8b9c1e76cb2f7")
V2_HEADER = ("canonical-upstream-suite-handoff", "verified-canonical-suite-receipt-unadmitted",
             "khronos-default-mustpass-broader-than-vulkan-1.4-core", "forbidden",
             "outside-admitted-implementation-source-closure")
V2_STATE = (False, False, False)
class ReconcileError(ValueError):
    """A current proof input is incomplete, mixed, stale, or falsely ready."""
@dataclass(frozen=True)
class CurrentEvidence:
    roles: tuple[tuple[str, str, str, str], ...]
    blockers: tuple[tuple[str, str], ...]
    first_blocker: tuple[str, str]
    inventory_sha256: str
    v1_digests: tuple[tuple[str, str], ...]
    v1_v2_bridge: tuple[str, str, str]
    v2_digests: tuple[tuple[str, str], ...]
    member_count: int
    member_total_bytes: int
    selector_scope: str
    docs_generated_artifacts: str
    v2_state: tuple[bool, bool, bool]
    state: str = field(default="blocked", init=False)
    admission_eligible: bool = field(default=False, init=False)
    inventory_ready: bool = field(default=False, init=False)
    fresh_cache_ready: bool = field(default=False, init=False)
    cutover_ready: bool = field(default=False, init=False)
    f03_ready: bool = field(default=False, init=False)
def reject(message: str) -> None:
    raise ReconcileError(message)
def paths(value: object, count: int, label: str) -> tuple[Path, ...]:
    if not isinstance(value, tuple) or len(value) != count or any(not isinstance(item, Path) for item in value):
        reject(f"{label} paths are incomplete")
    return value
def hashes(value: tuple[Path, ...]) -> tuple[str, ...]:
    try:
        return tuple(hashlib.sha256(path.read_bytes()).hexdigest() for path in value)
    except OSError as error:
        reject(f"immutable evidence cannot be read: {error}")
def current_contract(contract_path: Path, requirements_path: Path):
    try:
        frozen = PROOF.validate(contract_path, requirements_path)
        requirements, _ = PROOF.load(requirements_path, "source requirements")
        inventory = PROOF.locked_digest(requirements.get("inventory_sha256"), "source requirements inventory lock")
    except PROOF.ContractError as error:
        reject(f"proof contract failed: {error}")
    return frozen, inventory
def current_v1(value: tuple[Path, ...], audits: object):
    checked = paths(value, 3, "V1")
    bundle = dict(PRE.TRANSITION.MAP.AUDITS) if audits is None else audits
    try:
        pre = PRE.validate(*checked, bundle)
    except (PRE.AggregateError, TypeError) as error:
        reject(f"V1 evidence failed: {error}")
    locked = hashes((*checked, *(bundle[profile] for profile in V1_PROFILES)))
    if locked != V1_LOCK:
        reject("V1 immutable predecessor bundle drifted")
    return pre, tuple(zip(V1_NAMES, locked)), checked, bundle
def current_v2(value: tuple[Path, ...]):
    try:
        checked = paths(value, 7, "V2")
        suite = V2.validate(*checked)
        root, _, _, live, _ = V2.inputs(*checked[1:])
        return suite, root, live
    except (V2.HandoffError, TypeError) as error:
        reject(f"V2 evidence failed: {error}")
def bridge(source_path: Path, audit_path: Path, root: object, root_sha256: str) -> tuple[str, str, str]:
    try:
        source, audit = PRE.document(source_path, "source map"), PRE.document(audit_path, "Vulkan candidate audit")
    except PRE.AggregateError as error:
        reject(f"V1/V2 bridge failed: {error}")
    shapes, candidates = source.get("shapes"), audit.get("candidates")
    source_root = shapes[-1] if isinstance(shapes, list) and shapes else {}
    candidate = next((row for row in candidates if isinstance(row, dict)
                      and row.get("required_input_id") == root.suite_id), {}) if isinstance(candidates, list) else {}
    entry = candidate.get("entry") if isinstance(candidate, dict) else {}
    actual = (source_root.get("root_selector") if isinstance(source_root, dict) else None,
              candidate.get("selector") if isinstance(candidate, dict) else None,
              entry.get("revision") if isinstance(entry, dict) else None,
              entry.get("sha256") if isinstance(entry, dict) else None)
    if actual != (root.root_path, root.root_path, root.peeled_commit, root_sha256):
        reject("V1/V2 selector, revision, or root identity is detached")
    return root.root_path, root.peeled_commit, root_sha256
def validate(contract_path: Path = PROOF.CONTRACT, requirements_path: Path = PROOF.REQUIREMENTS,
             v1_paths: tuple[Path, ...] = V1_PATHS, audits: object = None,
             v2_paths: tuple[Path, ...] = V2_PATHS) -> CurrentEvidence:
    frozen, requirement_inventory = current_contract(contract_path, requirements_path)
    pre, v1_digests, checked_v1, bundle = current_v1(v1_paths, audits)
    suite, root, live = current_v2(v2_paths)
    roles = tuple(row[:3] for row in frozen.roles)
    ids = tuple(row[2] for row in frozen.roles)
    v1_state = (pre.state, pre.admission_eligible, pre.cutover_ready)
    if (roles != PROOF.CANONICAL or ids != pre.required_input_ids or requirement_inventory != pre.inventory_revision
            or pre.direct_candidate_ids != PRE.DIRECT or pre.blockers != PRE.BLOCKERS
            or tuple((row.required_input_id, row.observation_count) for row in pre.vulkan) != PRE.VULKAN
            or v1_state != ("pre-admission", False, False) or root.suite_id != ids[-1]
            or live.get("root_sha256") != pre.vulkan[-1].root_sha256):
        reject("V1 evidence does not retain one exact pre-admission predecessor")
    v1_v2_bridge = bridge(checked_v1[1], bundle["vulkan-1.4-core"], root, pre.vulkan[-1].root_sha256)
    header = tuple(suite.get(key) for key in ("kind", "status", "selector_scope", "local_filtering", "docs_generated_artifacts"))
    state = tuple(suite.get(key) for key in ("admitted", "cutover_ready", "satisfies_vulkan_14_core_manifest"))
    identities = tuple((key, suite.get(key)) for key in V2_DIGESTS)
    if (header != V2_HEADER or state != V2_STATE or tuple(value for _, value in identities) != V2_LOCK
            or (suite.get("member_count"), suite.get("member_total_bytes")) != (98, 434669348)):
        reject("V2 evidence does not retain the canonical unadmitted suite")
    reasons = dict(pre.blockers)
    blocked = tuple((identifier, reasons[identifier]) for identifier in ids if identifier in reasons)
    if blocked != pre.blockers or not blocked:
        reject("current blockers are not in canonical role order")
    return CurrentEvidence(frozen.roles, blocked, blocked[0], pre.inventory_revision, v1_digests, v1_v2_bridge, identities,
                           suite["member_count"], suite["member_total_bytes"], suite["selector_scope"],
                           suite["docs_generated_artifacts"], state)
def main() -> None:
    try:
        result = validate() if len(sys.argv) == 1 else None
        if result is None:
            raise SystemExit("usage: current_evidence_reconciler.py")
    except ReconcileError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"BLOCKED: {len(result.roles)} roles, first {result.first_blocker[0]} {result.first_blocker[1]}")
if __name__ == "__main__":
    main()
