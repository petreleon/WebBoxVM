"""Read-only semantic joins to the reviewed V1 Docs predecessor evidence."""

from __future__ import annotations

import hashlib
import importlib.util
import sys
from pathlib import Path

from successor_identity_model import ADAPTER_FIELDS, PredecessorAdapter, reject
from successor_identity_parse import digest, document, text

PATH_NAMES = frozenset(("rules", "audit", "boundary"))
HERE = Path(__file__).resolve().parent
V1_SCHEMA = HERE.parents[2] / "04-post-cutover-rules/post_cutover_schema.py"
AUDIT_CONTRACT = HERE.parents[4] / "candidate_contract.py"
BOUNDARY_CONTRACT = HERE.parents[2] / "03-vulkan-boundaries/boundary_contract.py"
RULE_FIELDS = frozenset((
    "schema", "phase", "source_shapes", "candidate_audit", "atomic_cutover",
    "future_artifacts", "renew_consumers",
))
AUDIT_FIELDS = frozenset(("schema", "profile", "inventory_sha256", "candidates"))
CANDIDATE_FIELDS = frozenset((
    "required_input_id", "role", "decision", "entry", "selector",
    "selector_case_count", "coverage", "admission_blocker",
))
BOUNDARY_FIELDS = frozenset(("schema", "profile", "boundaries"))
DOCS_BOUNDARY_FIELDS = frozenset((
    "profile", "role", "required_input_id", "shape", "state", "root_sha256",
    "blocker", "root_fallback", "admission_requirements", "scope_exclusions",
    "observation",
))
DOCS_REQUIREMENTS = (
    "generated-and-transitive-members-pinned",
    "macro-and-conditional-configuration-pinned",
    "core-only-scope-bounded",
)


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


V1 = reviewed_module("f024_successor_v1_schema", V1_SCHEMA)
AUDIT = reviewed_module("f024_successor_vulkan_audit", AUDIT_CONTRACT)
BOUNDARY = reviewed_module("f024_successor_vulkan_boundaries", BOUNDARY_CONTRACT)


def predecessor_digests(paths: object) -> dict[str, str]:
    if not isinstance(paths, dict) or set(paths) != PATH_NAMES:
        reject("predecessor paths are incomplete")
    result = {}
    for name, path in paths.items():
        if not isinstance(path, Path):
            reject("predecessor path is not a Path")
        try:
            result[name] = hashlib.sha256(path.read_bytes()).hexdigest()
        except OSError as error:
            reject(f"predecessor {name} cannot be read: {error}")
    return result


def transition_rules(path: Path) -> None:
    value = document(path)
    if (set(value) != RULE_FIELDS or type(value.get("schema")) is not int
            or value.get("schema") != 1 or value.get("phase") != "transition-design"
            or value.get("source_shapes") != list(V1.SOURCE_SHAPES)
            or value.get("candidate_audit") != V1.CANDIDATE_AUDIT
            or value.get("atomic_cutover") != V1.ATOMIC_CUTOVER
            or value.get("future_artifacts") != list(V1.FUTURE_ARTIFACTS)
            or value.get("renew_consumers") != list(V1.CONSUMERS)):
        reject("predecessor rules are not the exact V1 transition-design")


def rejected_root(path: Path) -> tuple[str, str, str]:
    try:
        decisions = AUDIT.validate(path, "vulkan-1.4-core")
    except AUDIT.AuditError as error:
        reject(f"predecessor audit fails exact F02.2 validation: {error}")
    if decisions != ("rejected", "rejected"):
        reject("predecessor audit weakens the reviewed Vulkan decisions")
    audit = document(path)
    if (set(audit) != AUDIT_FIELDS or type(audit.get("schema")) is not int
            or audit.get("schema") != 1 or audit.get("profile") != "vulkan-1.4-core"):
        reject("predecessor audit is not the V1 Vulkan profile")
    rows = audit.get("candidates")
    selected = [item for item in rows if isinstance(item, dict)
                and item.get("required_input_id") == "vulkan-14-spec"] if isinstance(rows, list) else []
    if len(selected) != 1 or set(selected[0]) != CANDIDATE_FIELDS:
        reject("predecessor audit has no unique Docs candidate")
    row, entry = selected[0], selected[0].get("entry")
    expected = ("vulkan-14-spec", "api-limit-format-spec", "rejected", "compound-unadmitted",
                "vulkan-docs-core-generated-closure-unadmitted")
    if tuple(row.get(key) for key in (
            "required_input_id", "role", "decision", "coverage", "admission_blocker")) != expected:
        reject("predecessor audit weakens the rejected Docs candidate")
    if not isinstance(entry, dict) or entry.get("id") != row["required_input_id"]:
        reject("predecessor audit root identity is malformed")
    return row["required_input_id"], digest(entry.get("sha256"), "predecessor root"), row["role"]


def docs_boundary(path: Path, audit_path: Path, root_id: str, root_digest: str, role: str) -> None:
    audits = dict(BOUNDARY.MAP.AUDITS, **{"vulkan-1.4-core": audit_path})
    try:
        records = BOUNDARY.validate(path, audits=audits)
    except BOUNDARY.BoundaryError as error:
        reject(f"predecessor boundary fails exact V1 validation: {error}")
    if tuple((item.required_input_id, item.state) for item in records) != (
            ("vulkan-14-spec", "unadmitted"), ("vulkan-cts-mustpass", "unadmitted")):
        reject("predecessor boundary weakens reviewed Vulkan states")
    value = document(path)
    if (set(value) != BOUNDARY_FIELDS or type(value.get("schema")) is not int
            or value.get("schema") != 1 or value.get("profile") != "vulkan-1.4-core"):
        reject("predecessor boundary is not the V1 Vulkan profile")
    rows = value.get("boundaries")
    selected = [item for item in rows if isinstance(item, dict)
                and item.get("required_input_id") == root_id] if isinstance(rows, list) else []
    if len(selected) != 1 or set(selected[0]) != DOCS_BOUNDARY_FIELDS:
        reject("predecessor boundary has no unique Docs root")
    row = selected[0]
    expected = ("vulkan-1.4-core", role, root_id, "unresolved-root", "unadmitted", "forbidden", root_digest)
    if tuple(row.get(key) for key in (
            "profile", "role", "required_input_id", "shape", "state", "root_fallback", "root_sha256")) != expected:
        reject("predecessor boundary weakens Docs root pre-admission")
    if (row.get("blocker") != "vulkan-docs-core-generated-closure-unadmitted"
            or row.get("admission_requirements") != list(DOCS_REQUIREMENTS)):
        reject("predecessor boundary weakens Docs admission requirements")


def adapter(value: object, paths: object) -> PredecessorAdapter:
    if not isinstance(value, dict) or set(value) != ADAPTER_FIELDS:
        reject("predecessor adapter has an invalid schema")
    if value.get("kind") != "v1-predecessor-only" or value.get("predecessor_decision") != "rejected":
        reject("predecessor adapter attempts successor state")
    actual = predecessor_digests(paths)
    names = {
        "predecessor_rules_sha256": "rules", "predecessor_audit_sha256": "audit",
        "predecessor_boundary_sha256": "boundary",
    }
    for field, name in names.items():
        if digest(value.get(field), field) != actual[name]:
            reject("predecessor adapter has a stale or mixed digest")
    transition_rules(paths["rules"])
    root_id, root_digest, role = rejected_root(paths["audit"])
    docs_boundary(paths["boundary"], paths["audit"], root_id, root_digest, role)
    candidate_digest = digest(value.get("candidate_root_sha256"), "candidate root")
    if value.get("candidate_root_id") != root_id or candidate_digest != root_digest:
        reject("predecessor adapter does not preserve the rejected root")
    return PredecessorAdapter(root_id, root_digest, actual["rules"], actual["audit"],
                              actual["boundary"], value["predecessor_decision"])
