#!/usr/bin/env python3
"""Seal and verify one exact, non-admitting aggregate evidence result."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RECON_PATH = HERE.parent / "02-reconcile-current-evidence/current_evidence_reconciler.py"
RECEIPT = HERE / "blocked_state_receipt.json"
MAX_RECEIPT_BYTES = 64 * 1024
FIELDS = frozenset(("schema", "kind", "profile", "state", "readiness", "roles", "blockers",
                    "first_blocker", "inventory_sha256", "validator_digests", "v1_digests",
                    "v1_v2_bridge", "v2_digests", "v2_member_count", "v2_member_total_bytes",
                    "selector_scope", "docs_generated_artifacts", "v2_state", "receipt_sha256"))
READINESS = ("admission_eligible", "inventory_ready", "fresh_cache_ready", "cutover_ready", "f03_ready")
V2_STATE = (False, False, False)


class ReceiptError(ValueError):
    """A blocked-state receipt is malformed, stale, or falsely ready."""


def reject(message: str) -> None:
    raise ReceiptError(message)


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


RECON = reviewed("f024_blocked_receipt_reconciler", RECON_PATH)


def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value:
            reject("receipt has a duplicate JSON key")
        value[key] = item
    return value


def sha256(path: Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        reject(f"receipt input cannot be read: {error}")


def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "receipt_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def alike(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right


def document(path: Path) -> dict[str, object]:
    try:
        raw = path.read_bytes()
        if len(raw) > MAX_RECEIPT_BYTES:
            reject("receipt exceeds its JSON byte limit")
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
    except ReceiptError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"receipt cannot be read: {error}")
    if not isinstance(value, dict) or set(value) != FIELDS or type(value.get("schema")) is not int:
        reject("receipt has an unexpected schema")
    return value


def validators() -> dict[str, str]:
    return {"reconciler_sha256": sha256(RECON_PATH), "proof_contract_sha256": sha256(RECON.PROOF.CONTRACT),
            "source_requirements_sha256": sha256(RECON.PROOF.REQUIREMENTS),
            "pre_admission_validator_sha256": sha256(Path(RECON.PRE.__file__)),
            "v2_handoff_validator_sha256": sha256(Path(RECON.V2.__file__))}


def rows(value: tuple[tuple[str, ...], ...]) -> list[list[str]]:
    return [list(row) for row in value]


def build(reconciler_args: object = None) -> dict[str, object]:
    if reconciler_args is not None and not isinstance(reconciler_args, dict):
        reject("reconciler arguments are invalid")
    try:
        result = RECON.validate(**({} if reconciler_args is None else reconciler_args))
    except (RECON.ReconcileError, TypeError) as error:
        reject(f"current evidence failed: {error}")
    ready = {key: getattr(result, key) for key in READINESS}
    if (result.state, tuple(ready.values()), result.v2_state, len(result.roles), len(result.blockers),
            result.first_blocker, result.member_count, result.member_total_bytes) != (
            "blocked", (False,) * len(READINESS), V2_STATE, 6, 3,
            ("gles-cts-manifest", "requires-multifile-core-selector-closure"), 98, 434669348):
        reject("current evidence is not the exact blocked diagnostic state")
    value: dict[str, object] = {
        "schema": 1, "kind": "blocked-admission-state-receipt",
        "profile": "proof-of-correctly-blocked-admission-state", "state": result.state,
        "readiness": ready, "roles": rows(result.roles), "blockers": rows(result.blockers),
        "first_blocker": list(result.first_blocker), "inventory_sha256": result.inventory_sha256,
        "validator_digests": validators(), "v1_digests": rows(result.v1_digests),
        "v1_v2_bridge": list(result.v1_v2_bridge), "v2_digests": rows(result.v2_digests),
        "v2_member_count": result.member_count, "v2_member_total_bytes": result.member_total_bytes,
        "selector_scope": result.selector_scope, "docs_generated_artifacts": result.docs_generated_artifacts,
        "v2_state": list(result.v2_state), "receipt_sha256": "",
    }
    value["receipt_sha256"] = digest(value)
    return value


def validate(path: Path = RECEIPT, reconciler_args: object = None) -> dict[str, object]:
    value = document(path)
    if not isinstance(value.get("receipt_sha256"), str) or value["receipt_sha256"] != digest(value):
        reject("receipt self-hash is invalid")
    expected = build(reconciler_args)
    if not alike(value, expected):
        reject("receipt does not bind exact current blocked evidence")
    return value


def main() -> None:
    try:
        value = build() if sys.argv[1:] == ["--build"] else validate() if len(sys.argv) == 1 else None
        if value is None:
            raise SystemExit("usage: blocked_state_receipt.py [--build]")
    except ReceiptError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    if sys.argv[1:] == ["--build"]:
        print(json.dumps(value, indent=2, sort_keys=True))
    else:
        print(f"RECEIPT: {value['state']} {len(value['blockers'])} blockers {value['receipt_sha256']}")


if __name__ == "__main__":
    main()
