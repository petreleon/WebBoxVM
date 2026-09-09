#!/usr/bin/env python3
"""Emit a fixture-only Venus provenance marker, never Venus protocol code."""

from __future__ import annotations

import argparse
from pathlib import Path

INPUT_ID = "venus-protocol-registry"
INPUT_SHA256 = "d92839bc728fa9ad9a7decdc6b91df6fa1a0fb26cffae4009865f18a789e0535"
MARKER = (
    "# Fixture-only Venus provenance marker; not protocol or runtime code.\n"
    "format=webboxvm-fixture-v1\n"
    f"input-id={INPUT_ID}\n"
    f"input-sha256={INPUT_SHA256}\n"
).encode()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input-id", required=True)
    parser.add_argument("--input-sha256", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if (args.input_id, args.input_sha256) != (INPUT_ID, INPUT_SHA256):
        parser.error("fixture marker accepts only the pinned Venus registry identity")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(MARKER)


if __name__ == "__main__":
    main()
