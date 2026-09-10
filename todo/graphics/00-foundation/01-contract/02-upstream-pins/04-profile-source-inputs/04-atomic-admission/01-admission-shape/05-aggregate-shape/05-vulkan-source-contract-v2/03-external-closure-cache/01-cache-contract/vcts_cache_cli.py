#!/usr/bin/env python3
"""Populate or offline-verify the external V2 Vulkan CTS closure cache."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from vcts_cache_contract import io, populate, repository_root, verify

HERE = Path(__file__).resolve().parent
SCHEMA = HERE.parent.parent / "02-canonical-suite-schema"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--identity", type=Path, default=SCHEMA / "vcts_root_identity.json")
    parser.add_argument("--timeout", type=float, default=60.0)
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    try:
        repository = repository_root(HERE)
        receipt = (verify(args.identity, args.ledger, args.cache_root, repository) if args.verify
                   else populate(args.identity, args.ledger, args.cache_root, repository, args.timeout))
    except io.CacheError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    state = "reused" if receipt.reused else "fetched"
    print(f"PASS: {state} {receipt.member_count} members {receipt.total_bytes} bytes {receipt.digest}")


if __name__ == "__main__":
    main()
