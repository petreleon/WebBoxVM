#!/usr/bin/env python3
"""Run explicitly selected local checks from a profile-independent catalog."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from catalog import CatalogError, load_catalog, select_checks
from result import write_json
from runner import execute, invalid


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--catalog", type=Path, required=True)
    parser.add_argument("--select", action="append", default=[])
    parser.add_argument("--result", type=Path, required=True)
    args = parser.parse_args()
    root, catalog, result_path = args.root.resolve(), args.catalog.resolve(), args.result.resolve()
    try:
        checks = select_checks(load_catalog(catalog), args.select)
        record, exit_status = execute(root, catalog, checks, args.select)
    except (CatalogError, OSError, ValueError) as error:
        record, exit_status = invalid(root, catalog, args.select, error), 2
        print(f"graphics runner: {error}", file=sys.stderr)
    write_json(result_path, record)
    for check in record["checks"]:
        sys.stdout.write(check["output"]["stdout"])
        sys.stderr.write(check["output"]["stderr"])
    raise SystemExit(exit_status)


if __name__ == "__main__":
    main()
