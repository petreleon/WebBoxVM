#!/usr/bin/env python3
"""Fail-closed, offline validation for F02.4 profile-source audits."""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path, PurePosixPath
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
MANIFEST = HERE.parent / "01-input-inventory/manifest.toml"


def source_api_loader():
    """Load the reviewed audit adapter without a cached-basename substitution."""
    spec = importlib.util.spec_from_file_location("f024_audit_source_api", HERE / "audit_source_api.py")
    if spec is None or spec.loader is None:
        raise RuntimeError("candidate audit source adapter cannot load")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(spec.name, None)
        raise
    return module.source_api


source_api = source_api_loader()
FAMILIES, load_inventory, ContractError, SourceInput = source_api()
TARGETS = {
    "opengl-4.6-core": (("opengl-46-core-spec", "api-limit-format-spec"),
                          ("opengl-cts-manifest", "conformance-manifest")),
    "gles-3.2": (("gles-32-spec", "api-limit-format-spec"),
                   ("gles-cts-manifest", "conformance-manifest")),
    "vulkan-1.4-core": (("vulkan-14-spec", "api-limit-format-spec"),
                          ("vulkan-cts-mustpass", "conformance-manifest")),
}
ROOT_FIELDS = frozenset(("schema", "profile", "inventory_sha256", "candidates"))
CANDIDATE_FIELDS = frozenset(("required_input_id", "role", "decision", "entry", "selector",
                              "selector_case_count", "coverage", "admission_blocker"))


class AuditError(ValueError):
    """A profile-source audit has an unsafe or unreviewable candidate."""


def reject(message: str) -> None:
    raise AuditError(message)


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"candidate audit cannot be read: {error}")
    if not isinstance(value, dict):
        reject("candidate audit is not a JSON object")
    return value


def selector(value: object, source_url: str) -> str:
    if not isinstance(value, str) or not value:
        reject("candidate selector is empty")
    path = PurePosixPath(value)
    if path.is_absolute() or str(path) != value or ".." in path.parts:
        reject("candidate selector is unsafe")
    if not urlsplit(source_url).path.endswith("/" + value):
        reject("candidate selector does not name the pinned source")
    return value


def source(entry: object):
    try:
        return SourceInput.from_manifest(entry)
    except ContractError as error:
        reject(f"candidate violates F02.2 policy: {error}")


def candidate(value: object, expected: tuple[str, str], seen_families: set[str]) -> str:
    if not isinstance(value, dict) or set(value) != CANDIDATE_FIELDS:
        reject("candidate has an unexpected schema")
    identifier, role = expected
    if value["required_input_id"] != identifier or value["role"] != role:
        reject("candidate does not match the reviewed requirement")
    item = source(value["entry"])
    if item.identifier != identifier:
        reject("candidate entry id does not match the required input")
    family = value["entry"].get("source_family") if isinstance(value["entry"], dict) else None
    if family in FAMILIES or family in seen_families:
        reject("candidate reuses an existing or duplicate source family")
    if not isinstance(family, str):
        reject("candidate source family is invalid")
    seen_families.add(family)
    selector(value["selector"], item.url)
    count, decision, coverage, blocker = (value[field] for field in
                                          ("selector_case_count", "decision", "coverage", "admission_blocker"))
    if type(count) is not int or count < 0:
        reject("candidate selector_case_count is invalid")
    if (role == "conformance-manifest") != (count > 0):
        reject("candidate selector count does not match its role")
    if decision not in ("accepted", "rejected") or coverage not in ("complete-single-file", "compound-unadmitted"):
        reject("candidate has an invalid decision or coverage")
    if not isinstance(blocker, str):
        reject("candidate admission_blocker is invalid")
    if decision == "accepted" and (coverage != "complete-single-file" or blocker):
        reject("accepted candidate must be a complete unblocked single source")
    if decision == "rejected" and (coverage != "compound-unadmitted" or not blocker):
        reject("rejected candidate must expose a compound-source blocker")
    return decision


def validate(path: Path, profile: str) -> tuple[str, ...]:
    expected = TARGETS.get(profile)
    if expected is None:
        reject("candidate audit has an unknown target profile")
    value = document(path)
    if set(value) != ROOT_FIELDS or value.get("schema") != 1 or value.get("profile") != profile:
        reject("candidate audit does not match schema version 1")
    try:
        revision = load_inventory(MANIFEST).revision
    except Exception as error:
        reject(f"reviewed inventory cannot be loaded: {error}")
    if value.get("inventory_sha256") != revision:
        reject("candidate audit has a stale inventory_sha256")
    records = value.get("candidates")
    if not isinstance(records, list) or len(records) != len(expected):
        reject("candidate audit does not have exactly the required candidates")
    families: set[str] = set()
    return tuple(candidate(record, requirement, families) for record, requirement in zip(records, expected))


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: candidate_contract.py AUDIT.json PROFILE")
    try:
        decisions = validate(Path(sys.argv[1]), sys.argv[2])
    except AuditError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"AUDIT: {sys.argv[2]} " + ", ".join(decisions))


if __name__ == "__main__":
    main()
