#!/usr/bin/env python3
"""Public, one-flow entry points for unadmitted Docs output witnesses."""

from __future__ import annotations

import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "02-stage-core-inputs"
STAGE = HERE.parent / "01-stage-contract"
for directory in (INPUT, STAGE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

from vulkan_docs_input_contract import verify_staged_inputs
from vulkan_docs_output_store import _stage, _verify
from vulkan_docs_stage_bind import build_plan as _build_plan

__all__ = ("stage_outputs", "verify_staged_outputs")


def stage_outputs(observation: Path, artifact_root: Path, cache_root: Path) -> dict[str, object]:
    """Stage only after the exact prerequisite input snapshot verifies."""
    inputs = verify_staged_inputs(observation, artifact_root, cache_root)
    return _stage(_build_plan(observation, artifact_root, cache_root), artifact_root, inputs)


def verify_staged_outputs(observation: Path, artifact_root: Path, cache_root: Path) -> dict[str, object]:
    inputs = verify_staged_inputs(observation, artifact_root, cache_root)
    return _verify(_build_plan(observation, artifact_root, cache_root), inputs)


def main() -> None:
    if len(sys.argv) != 5 or sys.argv[1] not in ("stage", "verify"):
        raise SystemExit("usage: vulkan_docs_output_contract.py {stage|verify} OBSERVATION.json ARTIFACT_ROOT EXTERNAL_CACHE_ROOT")
    try:
        command = stage_outputs if sys.argv[1] == "stage" else verify_staged_outputs
        receipt = command(Path(sys.argv[2]), Path(sys.argv[3]), Path(sys.argv[4]))
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    outputs = receipt["output_witnesses"]
    print(f"OUTPUT-STAGE: staging-only-unadmitted, {len(outputs)} witnesses, {sum(row['file_count'] for row in outputs)} files, 0 markers")


if __name__ == "__main__":
    main()
