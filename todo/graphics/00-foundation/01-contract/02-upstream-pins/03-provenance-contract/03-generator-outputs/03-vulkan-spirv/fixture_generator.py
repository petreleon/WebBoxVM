#!/usr/bin/env python3
"""Generate tiny provenance-only fixtures; this emits no protocol implementation."""

from __future__ import annotations

import argparse
from pathlib import Path

NAME = "webboxvm-provenance-fixture-generator"
VERSION = "1"
INPUTS = {
    "vulkan-registry": (
        "vulkan-registry",
        "cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06",
    ),
    "spirv-core-grammar": (
        "spirv-core-grammar",
        "db8581272b63d232268094a47b68d18a0464fc911e06004d57419924fe660ba4",
    ),
}


def render(family: str) -> bytes:
    """Return a stable marker derived from an identity, never upstream source bytes."""
    identifier, input_sha256 = INPUTS[family]
    return (
        f"generator={NAME}/{VERSION}\n"
        f"family={family}\n"
        f"input_id={identifier}\n"
        f"input_sha256={input_sha256}\n"
        "scope=provenance-fixture-only;not-protocol-code\n"
    ).encode("utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("family", choices=sorted(INPUTS))
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()
    arguments.output.write_bytes(render(arguments.family))


if __name__ == "__main__":
    main()
