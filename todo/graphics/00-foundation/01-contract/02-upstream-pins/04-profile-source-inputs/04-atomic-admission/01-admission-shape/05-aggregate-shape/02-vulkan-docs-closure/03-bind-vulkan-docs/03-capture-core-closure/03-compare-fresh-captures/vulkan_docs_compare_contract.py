#!/usr/bin/env python3
"""Validate the compact receipt for one independently bound Docs capture pair."""

from __future__ import annotations

import sys
from pathlib import Path

from vulkan_docs_compare_bind import bind_pair
from vulkan_docs_compare_model import COMPARISON_FIELDS, STATUS, CaptureComparison, reject
from vulkan_docs_compare_parse import canonical, digest, document, exact_object


def _receipt_value(value: object, comparison: CaptureComparison) -> CaptureComparison:
    data = exact_object(value, COMPARISON_FIELDS, "capture comparison receipt")
    if tuple(data.get(name) for name in ("schema", "contract", "status", "profile", "role", "required_input_id")) != (
            1, "vulkan-docs-core-capture-comparison-v1", STATUS, "vulkan-1.4-core", "api-limit-format-spec", "vulkan-14-spec"):
        reject("capture comparison receipt attempts an admitted or unrelated state")
    actual = digest(data.get("comparison_sha256"), "capture comparison receipt sha256")
    payload = dict(data)
    payload.pop("comparison_sha256", None)
    if actual != canonical(payload, "webboxvm-graphics-vulkan-docs-core-capture-comparison-v1"):
        reject("capture comparison receipt has a stale self identity")
    if data != comparison.value:
        reject("capture comparison receipt does not bind the selected raw artifacts")
    return comparison


def _receipt_value_fixture(value: object, comparison: CaptureComparison) -> CaptureComparison:
    """Test-only receipt comparator; receipt() always binds live pinned artifacts."""
    return _receipt_value(value, comparison)


def receipt(path: Path, observation: Path, artifact_root: Path) -> CaptureComparison:
    return _receipt_value(document(path), bind_pair(observation, artifact_root))


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit("usage: vulkan_docs_compare_contract.py RECEIPT.json OBSERVATION.json ARTIFACT_ROOT")
    try:
        result = receipt(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]))
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"COMPARISON: {result.state}, {result.value['counts']['capture_count']} captures, 0 cutover-ready")


if __name__ == "__main__":
    main()
