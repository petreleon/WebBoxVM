#!/usr/bin/env python3
"""Fetch declared immutable sources into a caller-supplied external cache."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from source_cache import fetch_to_cache
from source_model import ContractError, ExternalCache, load_manifest, repository_root

HERE = Path(__file__).resolve().parent
DEFAULT_MANIFEST = HERE.parents[1] / "01-input-inventory" / "manifest.toml"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=30.0)
    args = parser.parse_args()
    try:
        if args.timeout <= 0:
            raise ContractError("timeout must be positive")
        cache = ExternalCache.from_path(args.cache_root, repository_root(HERE))
        for source in load_manifest(args.manifest):
            path, reused = fetch_to_cache(cache, source, args.timeout)
            state = "reused" if reused else "fetched"
            print(f"PASS: {source.identifier} {state} {path}")
    except ContractError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
