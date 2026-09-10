#!/usr/bin/env python3
"""Validate the compact unadmitted receipt for one bound core input scope."""

from __future__ import annotations

import sys
from pathlib import Path

from vulkan_docs_scope_bind import bind_capture
from vulkan_docs_scope_model import RECEIPT_FIELDS, ScopeManifest, reject
from vulkan_docs_scope_parse import canonical, digest, document, exact_object


def compact(scope: ScopeManifest) -> dict[str, object]:
    value = scope.value
    raw = value["raw_records"]
    derived = value["derived_records"]
    receipt = {
        "schema": 1, "contract": "vulkan-docs-core-input-scope-v1", "status": "input-scope-only-unadmitted",
        "profile": "vulkan-1.4-core", "role": "api-limit-format-spec", "required_input_id": "vulkan-14-spec",
        "observation_sha256": scope.capture.observation_digest,
        "capture": value["capture"], "build_witness_sha256": value["build_witness_sha256"],
        "configuration_sha256": value["configuration_sha256"], "producer": value["producer"],
        "inputs": {
            "raw_count": len(raw), "derived_count": len(derived),
            "raw_identity_sha256": canonical(raw, "webboxvm-graphics-vulkan-docs-core-raw-identities-v1"),
            "derived_identity_sha256": canonical(derived, "webboxvm-graphics-vulkan-docs-core-derived-identities-v1"),
            "input_manifest_sha256": scope.capture.input_manifest_digest,
        },
        "includes": {"count": len(value["includes"]), "identity_sha256": scope.capture.include_digest},
        "conditions": value["conditions"], "scope_identity_sha256": scope.scope_digest,
        "manifest_sha256": scope.manifest_digest,
    }
    if set(receipt) != RECEIPT_FIELDS - {"scope_receipt_sha256"}:
        reject("core scope receipt cannot be sealed from an invalid schema")
    receipt["scope_receipt_sha256"] = canonical(receipt, "webboxvm-graphics-vulkan-docs-core-scope-receipt-v1")
    return receipt


def receipt_value(value: object, scope: ScopeManifest) -> ScopeManifest:
    data = exact_object(value, RECEIPT_FIELDS, "core scope receipt")
    if tuple(data.get(name) for name in ("schema", "contract", "status", "profile", "role", "required_input_id")) != (
            1, "vulkan-docs-core-input-scope-v1", "input-scope-only-unadmitted", "vulkan-1.4-core",
            "api-limit-format-spec", "vulkan-14-spec"):
        reject("core scope receipt attempts an admitted or unrelated state")
    actual = digest(data.get("scope_receipt_sha256"), "core scope receipt sha256")
    payload = dict(data)
    payload.pop("scope_receipt_sha256", None)
    if actual != canonical(payload, "webboxvm-graphics-vulkan-docs-core-scope-receipt-v1"):
        reject("core scope receipt has a stale self identity")
    if data != compact(scope):
        reject("core scope receipt does not bind the selected normalized capture")
    return scope


def receipt(path: Path, observation: Path, artifact_root: Path, run_id: str) -> ScopeManifest:
    return receipt_value(document(path), bind_capture(observation, artifact_root, run_id))


def main() -> None:
    if len(sys.argv) != 5:
        raise SystemExit("usage: vulkan_docs_scope_contract.py RECEIPT.json OBSERVATION.json ARTIFACT_ROOT RUN_ID")
    try:
        result = receipt(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]), sys.argv[4])
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"SCOPE: {result.state}, 0 cutover-ready")


if __name__ == "__main__":
    main()
