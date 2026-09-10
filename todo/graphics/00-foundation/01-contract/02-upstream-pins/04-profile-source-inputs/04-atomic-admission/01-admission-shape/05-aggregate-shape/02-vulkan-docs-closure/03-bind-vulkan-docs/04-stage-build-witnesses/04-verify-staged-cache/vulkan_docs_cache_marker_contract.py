#!/usr/bin/env python3
"""Public one-flow marker publication and cache-reuse entry points."""

from __future__ import annotations

import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
STAGE = HERE.parent / "01-stage-contract"
if str(STAGE) not in sys.path:
    sys.path.insert(0, str(STAGE))

from vulkan_docs_cache_marker_store import _publish, _reuse
from vulkan_docs_stage_bind import build_plan as _build_plan

__all__ = ("publish_staged_cache", "reuse_staged_cache")


def publish_staged_cache(observation: Path, artifact_root: Path, cache_root: Path) -> dict[str, object]:
    """Publish one marker only after rehashing the complete already-staged cache."""
    return _publish(_build_plan(observation, artifact_root, cache_root))


def reuse_staged_cache(observation: Path, artifact_root: Path, cache_root: Path) -> dict[str, object]:
    return _reuse(_build_plan(observation, artifact_root, cache_root))


def main() -> None:
    if len(sys.argv) != 5 or sys.argv[1] not in ("publish", "reuse"):
        raise SystemExit("usage: vulkan_docs_cache_marker_contract.py {publish|reuse} OBSERVATION.json ARTIFACT_ROOT EXTERNAL_CACHE_ROOT")
    try:
        command = publish_staged_cache if sys.argv[1] == "publish" else reuse_staged_cache
        marker = command(Path(sys.argv[2]), Path(sys.argv[3]), Path(sys.argv[4]))
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"CACHE-MARKER: {marker['status']}, {marker['inputs']['total_count']} inputs, "
          f"{len(marker['output_witnesses'])} witnesses, 0 cutover-ready")


if __name__ == "__main__":
    main()
