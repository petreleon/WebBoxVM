#!/usr/bin/env python3
"""Public, one-flow entry points for the unadmitted Docs input snapshot."""

from __future__ import annotations

import sys
from pathlib import Path

from vulkan_docs_input_model import reject
from vulkan_docs_input_store import _stage, _verify
from vulkan_docs_stage_bind import build_plan as _build_plan

__all__ = ("stage_inputs", "verify_staged_inputs")


def stage_inputs(observation: Path, artifact_root: Path, cache_root: Path) -> dict[str, object]:
    """Bind providers and cache root together; no public arbitrary-plan/root pairing exists."""
    return _stage(_build_plan(observation, artifact_root, cache_root), artifact_root)


def verify_staged_inputs(observation: Path, artifact_root: Path, cache_root: Path) -> dict[str, object]:
    return _verify(_build_plan(observation, artifact_root, cache_root))


def main() -> None:
    if len(sys.argv) != 5 or sys.argv[1] not in ("stage", "verify"):
        raise SystemExit("usage: vulkan_docs_input_contract.py {stage|verify} OBSERVATION.json ARTIFACT_ROOT EXTERNAL_CACHE_ROOT")
    try:
        command = stage_inputs if sys.argv[1] == "stage" else verify_staged_inputs
        receipt = command(Path(sys.argv[2]), Path(sys.argv[3]), Path(sys.argv[4]))
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    inputs = receipt["inputs"]
    print(f"INPUT-STAGE: staging-only-unadmitted, {inputs['raw_count']} raw, {inputs['derived_count']} derived, 0 markers")


if __name__ == "__main__":
    main()
