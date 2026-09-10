#!/usr/bin/env python3
"""Public fail-closed entry points for the actual Docs staging-plan contract."""

from __future__ import annotations

import sys
from pathlib import Path

from vulkan_docs_stage_bind import build_plan, verify_value
from vulkan_docs_stage_model import StageError, StagingPlan

__all__ = ("build_plan", "verify_plan", "StageError", "StagingPlan")


def verify_plan(value: object, observation: Path, artifact_root: Path, cache_root: Path) -> StagingPlan:
    """Compare decoded plan data to a freshly bound plan; cache readers stay descriptor-safe elsewhere."""
    return verify_value(value, build_plan(observation, artifact_root, cache_root))


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit("usage: vulkan_docs_stage_contract.py OBSERVATION.json ARTIFACT_ROOT EXTERNAL_CACHE_ROOT")
    try:
        result = build_plan(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]))
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    inputs = result.value["inputs"]
    print(
        f"STAGING-CONTRACT: {result.state}, {len(inputs['raw_records'])} raw, "
        f"{len(inputs['derived_records'])} derived, 0 payloads, 0 cutover-ready"
    )


if __name__ == "__main__":
    main()
