#!/usr/bin/env python3
"""Emit an ignored compact comparison artifact from the two pinned Docs observations."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from vulkan_docs_compare_bind import bind_pair


def write(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--observation", type=Path, required=True)
    parser.add_argument("--artifact-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    try:
        result = bind_pair(arguments.observation, arguments.artifact_root)
        write(arguments.output, result.value)
    except Exception as error:
        print(f"FAIL: {error}")
        raise SystemExit(2)
    print(f"COMPARISON: {result.state}, {result.value['counts']['capture_count']} captures, 0 cutover-ready")


if __name__ == "__main__":
    main()
