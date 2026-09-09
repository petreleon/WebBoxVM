#!/usr/bin/env python3
"""Generate and verify deterministic, provenance-bound graphics record chunks."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

import graphics_chunk_output as chunk_output
import graphics_chunk_schema as chunk_schema

MAX_LINES = 180
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]*$")
Record = tuple[str, tuple[str, ...]]
Records = tuple[Record, ...]
Profile = chunk_schema.ChunkSchema


class ChunkError(ValueError):
    """Raised when generated source cannot be safely accepted."""


def fail(message: str) -> None:
    raise ChunkError(message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_spec(path: Path) -> tuple[Profile, str, Records, str]:
    raw = path.read_bytes()
    try:
        document = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"invalid chunk specification: {error}")
    try:
        profile, revision, records = chunk_schema.spec_fields(document)
    except chunk_schema.ChunkSchemaError as error:
        fail(str(error))
    if not isinstance(records, list) or not records:
        fail("chunk specification must contain ordered records")
    parsed = []
    for record in records:
        if not isinstance(record, dict) or set(record) != {"id", "lines"}:
            fail("each record must contain only id and lines")
        identifier, lines = record["id"], record["lines"]
        if not isinstance(identifier, str) or not IDENTIFIER.fullmatch(identifier):
            fail("record has an invalid id")
        if not isinstance(lines, list) or not lines or any(
            not isinstance(line, str) or "\n" in line or "\r" in line for line in lines
        ):
            fail(f"record {identifier} has invalid source lines")
        parsed.append((identifier, tuple(lines)))
    identifiers = [identifier for identifier, _ in parsed]
    if identifiers != sorted(identifiers) or len(identifiers) != len(set(identifiers)):
        fail("records must be uniquely ordered by id")
    return profile, revision, tuple(parsed), sha256(raw)


def verify_source(path: Path, revision: str, profile: Profile) -> None:
    """Bind a specification to raw source bytes without parsing inventory composition."""
    if sha256(path.read_bytes()) != revision:
        fail(f"{profile.source_label} does not match chunk specification")


def verify_manifest(path: Path, revision: str) -> None:
    """Preserve the schema-v1 manifest verification API."""
    verify_source(path, revision, chunk_schema.V1)


def render_chunk(profile: Profile, index: int, revision: str, records: Records) -> bytes:
    first, last = records[0][0], records[-1][0]
    lines = [
        "# Generated graphics record chunk",
        f"# generator: {profile.generator}",
        f"# {profile.header_field}: {revision}",
        f"# record-range: {first}..{last}",
        f"# chunk-index: {index:04d}",
        "",
    ]
    for identifier, body in records:
        lines.extend((f"## {identifier}", *body, ""))
    text = "\n".join(lines)
    if len(text.splitlines()) > MAX_LINES:
        fail(f"chunk {index:04d} exceeds {MAX_LINES} physical lines")
    return text.encode("utf-8")


def build_chunks(profile: Profile, revision: str, records: Records) -> list[bytes]:
    chunks, pending = [], []
    for record in records:
        candidate = tuple((*pending, record))
        try:
            render_chunk(profile, len(chunks) + 1, revision, candidate)
        except ChunkError:
            if not pending:
                raise
            chunks.append(render_chunk(profile, len(chunks) + 1, revision, tuple(pending)))
            pending.clear()
            render_chunk(profile, len(chunks) + 1, revision, (record,))
        pending.append(record)
    chunks.append(render_chunk(profile, len(chunks) + 1, revision, tuple(pending)))
    return chunks


def expected_bundle(spec: Path) -> dict[str, bytes]:
    profile, revision, records, input_digest = load_spec(spec)
    chunks = build_chunks(profile, revision, records)
    bundle = {f"chunk-{index:04d}.md": chunk for index, chunk in enumerate(chunks, 1)}
    metadata = {
        "schema": profile.version,
        "generator": profile.generator,
        profile.identity_field: revision,
        "input_sha256": input_digest,
        "chunks": [
            {
                "name": name,
                "sha256": sha256(bundle[name]),
                "lines": len(bundle[name].decode().splitlines()),
            }
            for name in sorted(bundle)
        ],
    }
    bundle["metadata.json"] = (json.dumps(metadata, indent=2, sort_keys=True) + "\n").encode()
    return bundle


def write_bundle(output: Path, bundle: dict[str, bytes]) -> None:
    chunk_output.write_bundle(output, bundle, fail)


def check_bundle(output: Path, bundle: dict[str, bytes]) -> None:
    chunk_output.check_bundle(output, bundle, fail)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="atomically replace generated output")
    mode.add_argument("--check", action="store_true", help="reject stale generated output")
    parser.add_argument("--spec", type=Path, required=True)
    chunk_schema.add_source_arguments(parser)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        profile, revision, _, _ = load_spec(args.spec)
        source = chunk_schema.source_for(args, profile)
        verify_source(source, revision, profile)
        bundle = expected_bundle(args.spec)
        action = write_bundle if args.write else check_bundle
        action(args.output, bundle)
        status = "WROTE" if args.write else "PASS"
        metadata_digest = sha256(bundle["metadata.json"])
        print(f"{status}: {len(bundle) - 1} chunks; metadata sha256={metadata_digest}")
    except (ChunkError, chunk_schema.ChunkSchemaError, OSError) as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
