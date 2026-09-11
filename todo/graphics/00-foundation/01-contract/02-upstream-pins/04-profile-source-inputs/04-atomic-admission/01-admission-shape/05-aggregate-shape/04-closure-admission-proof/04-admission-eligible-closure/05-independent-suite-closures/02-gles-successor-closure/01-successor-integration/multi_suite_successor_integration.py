#!/usr/bin/env python3
"""Freeze a non-admitting aggregate future shape for Docs and GLES source wrappers."""

import hashlib, importlib.util, json, os, stat, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ELIGIBLE = HERE.parents[2]
SUITES = HERE.parents[1]
DOCS_DIR = ELIGIBLE / "06-successor-inventory-transition"
BOUNDARY_DIR = SUITES / "01-successor-boundary"
RECORD = HERE / "multi_suite_successor_integration.json"
MAX_BYTES = 64 * 1024
EFFECTS = ("active_inventory_changed", "cache_freshness_proved", "f03_changed", "admission_eligible",
           "admitted", "cutover_ready", "supported", "conformant", "certified", "near_native",
           "performance_claimed", "satisfies_vulkan_14_core_manifest")


class IntegrationError(ValueError):
    """A multi-suite successor plan is malformed, stale, or falsely admitting."""


def reject(message: str) -> None:
    raise IntegrationError(message)


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


if str(DOCS_DIR) not in sys.path:
    sys.path.insert(0, str(DOCS_DIR))
DOCS = reviewed("f025_multi_suite_docs", DOCS_DIR / "successor_inventory_transition.py")
BOUNDARY = reviewed("f025_multi_suite_boundary", BOUNDARY_DIR / "suite_successor_boundary.py")


def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value:
            reject("integration has a duplicate JSON key")
        value[key] = item
    return value


def raw(path: Path, label: str) -> bytes:
    try:
        if not stat.S_ISREG(path.lstat().st_mode):
            reject(f"{label} is not a regular file")
        descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(descriptor, "rb") as opened:
            if not stat.S_ISREG(os.fstat(descriptor).st_mode):
                reject(f"{label} is not a regular file")
            value = opened.read(MAX_BYTES + 1)
    except OSError as error:
        reject(f"{label} cannot be read: {error}")
    if len(value) > MAX_BYTES:
        reject(f"{label} exceeds its bounded size")
    return value


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(raw(path, "integration").decode("utf-8"), object_pairs_hook=pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"integration cannot be parsed: {error}")
    if not isinstance(value, dict):
        reject("integration is not an object")
    return value


def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "integration_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def alike(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right


def expected() -> dict[str, object]:
    try:
        docs_digest = DOCS.transition()
        inventory = DOCS.active_inventory()
        boundary = BOUNDARY.validate()
    except Exception as error:
        reject(f"reviewed predecessor is invalid: {error}")
    families = [item["source_family"] for item in inventory.inputs]
    gles = boundary["gles"]
    if (inventory.schema, inventory.revision, tuple(families), docs_digest,
            boundary["status"], boundary["effects"], gles["successor_closure_ready"]) != (
            2, DOCS.LOCK_SHA256, DOCS.ACTIVE_FAMILIES, docs_digest,
            "policy-only-unadmitted", {name: False for name in BOUNDARY.EFFECTS}, False):
        reject("reviewed Docs, GLES, or active predecessor has drifted")
    predecessor = {"active_schema": inventory.schema, "inventory_lock_sha256": inventory.revision,
                   "active_families": families, "active_inventory_changed": False}
    docs = {"transition_sha256": docs_digest, "successor_schema": 3, "family": "vulkan-docs",
            "independently_admitted": False, "complete_closure_required": True,
            "fresh_authority_and_lineage_required": True}
    gles_wrapper = {"boundary_policy_sha256": boundary["policy_sha256"], "profile": "gles-3.2",
                    "candidate_id": gles["candidate_id"], "candidate_sha256": gles["candidate_sha256"],
                    "family": "gles-cts", "core_member_count": len(gles["core_members"]),
                    "configuration_count": gles["core_configuration_count"],
                    "extension_exclusion_required": True, "per_member_max_bytes": 8 * 1024 * 1024,
                    "successor_closure_ready": False, "fresh_capture_required": True}
    aggregate = {"target_schema": 4, "wrapper_families": ["vulkan-docs", "gles-cts"],
                 "active_family_alias_allowed": False, "one_wrapper_per_family": True,
                 "root_and_nonroot_required": True, "independent_closures_required": True,
                 "cross_wrapper_substitution_allowed": False, "atomic_f02_f03_revalidation_required": True,
                 "active_mutation_permitted": False, "cache_freshness_proved": False}
    return {"predecessor": predecessor, "docs": docs, "gles": gles_wrapper, "aggregate": aggregate}


def validate(path: Path = RECORD) -> dict[str, object]:
    value = document(path)
    fields = {"schema", "contract", "status", "predecessor", "docs", "gles", "aggregate", "effects",
              "integration_sha256"}
    if (set(value) != fields or type(value.get("schema")) is not int
            or (value["schema"], value["contract"], value["status"]) != (
                4, "multi-suite-successor-integration-v4", "design-only-unadmitted")):
        reject("integration has an unexpected schema or status")
    if not isinstance(value.get("integration_sha256"), str) or value["integration_sha256"] != digest(value):
        reject("integration self-hash is invalid")
    if (not alike({key: value[key] for key in ("predecessor", "docs", "gles", "aggregate")}, expected())
            or not alike(value["effects"], {name: False for name in EFFECTS})):
        reject("integration does not bind the exact unadmitted multi-suite boundary")
    return value


def main() -> None:
    try:
        value = validate(Path(sys.argv[1]) if len(sys.argv) == 2 else RECORD) if len(sys.argv) < 3 else None
        if value is None:
            raise SystemExit("usage: multi_suite_successor_integration.py [INTEGRATION.json]")
    except IntegrationError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"INTEGRATION: {value['status']} {value['integration_sha256']}")


if __name__ == "__main__":
    main()
