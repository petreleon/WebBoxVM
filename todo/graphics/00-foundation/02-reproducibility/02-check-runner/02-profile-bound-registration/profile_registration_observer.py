#!/usr/bin/env python3
"""Add one F05.1 count marker only after an exact child command succeeds."""

from __future__ import annotations

import argparse
import subprocess
import sys


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--count", type=int, required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if args.count <= 0 or not command:
        raise SystemExit("usage: profile_registration_observer.py --count POSITIVE -- COMMAND...")
    try:
        result = subprocess.run(command, check=False)
    except OSError as error:
        print(f"observer cannot execute child: {error}", file=sys.stderr)
        raise SystemExit(127)
    if result.returncode:
        raise SystemExit(result.returncode)
    print(f"WEBBOXVM_GRAPHICS_OBSERVED_COUNT={args.count}")


if __name__ == "__main__":
    main()
