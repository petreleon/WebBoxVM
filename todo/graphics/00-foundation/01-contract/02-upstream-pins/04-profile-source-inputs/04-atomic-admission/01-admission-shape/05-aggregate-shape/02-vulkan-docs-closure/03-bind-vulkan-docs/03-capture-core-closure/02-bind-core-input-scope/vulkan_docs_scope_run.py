#!/usr/bin/env python3
"""Materialize ignored full core-scope artifacts from one pinned observation."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from vulkan_docs_scope_bind import bind_capture
from vulkan_docs_scope_contract import compact


def write(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--observation", type=Path, required=True)
    parser.add_argument("--artifact-root", type=Path, required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--receipt-output", type=Path, required=True)
    args = parser.parse_args()
    try:
        scope = bind_capture(args.observation, args.artifact_root, args.run_id)
        write(args.output, scope.value)
        write(args.receipt_output, compact(scope))
    except Exception as error:
        print(f"FAIL: {error}")
        raise SystemExit(2)
    print(f"SCOPE: {scope.state}, {len(scope.value['raw_records'])} raw, {len(scope.value['derived_records'])} derived")


if __name__ == "__main__":
    main()
