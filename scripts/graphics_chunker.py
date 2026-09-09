#!/usr/bin/env python3
"""Generate and verify deterministic, provenance-bound graphics record chunks."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import sys
import tempfile
from pathlib import Path

MAX_LINES = 180
GENERATOR = "webboxvm.graphics-chunker.v1"
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]*$")
DIGEST = re.compile(r"^[0-9a-f]{64}$")

class ChunkError(ValueError):
    """Raised when generated source cannot be safely accepted."""

def fail(message: str) -> None:
    raise ChunkError(message)

def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def physical_lines(text: str) -> int:
    return len(text.splitlines())

def load_spec(path: Path) -> tuple[str, tuple[tuple[str, tuple[str, ...]], ...], str]:
    raw = path.read_bytes()
    try:
        document = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"invalid chunk specification: {error}")
    if not isinstance(document, dict) or set(document) != {"schema", "manifest_revision", "records"}:
        fail("chunk specification must have schema, manifest_revision, and records")
    revision = document["manifest_revision"]
    records = document["records"]
    if type(document["schema"]) is not int or document["schema"] != 1 or not isinstance(revision, str) or not DIGEST.fullmatch(revision):
        fail("chunk specification has an invalid schema or manifest revision")
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
    return revision, tuple(parsed), sha256(raw)

def verify_manifest(path: Path, revision: str) -> None:
    if sha256(path.read_bytes()) != revision:
        fail("source manifest revision does not match chunk specification")

def render_chunk(index: int, revision: str, records: tuple[tuple[str, tuple[str, ...]], ...]) -> bytes:
    first, last = records[0][0], records[-1][0]
    lines = [
        "# Generated graphics record chunk",
        f"# generator: {GENERATOR}",
        f"# manifest-revision: {revision}",
        f"# record-range: {first}..{last}",
        f"# chunk-index: {index:04d}",
        "",
    ]
    for identifier, body in records:
        lines.extend((f"## {identifier}", *body, ""))
    text = "\n".join(lines)
    if physical_lines(text) > MAX_LINES:
        fail(f"chunk {index:04d} exceeds {MAX_LINES} physical lines")
    return text.encode("utf-8")

def build_chunks(revision: str, records: tuple[tuple[str, tuple[str, ...]], ...]) -> list[bytes]:
    chunks, pending = [], []
    for record in records:
        candidate = tuple(pending + [record])
        try:
            render_chunk(len(chunks) + 1, revision, candidate)
        except ChunkError:
            if not pending:
                raise
            chunks.append(render_chunk(len(chunks) + 1, revision, tuple(pending)))
            pending = [record]
            render_chunk(len(chunks) + 1, revision, tuple(pending))
        else:
            pending.append(record)
    chunks.append(render_chunk(len(chunks) + 1, revision, tuple(pending)))
    return chunks


def expected_bundle(spec: Path) -> dict[str, bytes]:
    revision, records, input_digest = load_spec(spec)
    chunks = build_chunks(revision, records)
    bundle = {f"chunk-{index:04d}.md": chunk for index, chunk in enumerate(chunks, 1)}
    metadata = {
        "schema": 1,
        "generator": GENERATOR,
        "manifest_revision": revision,
        "input_sha256": input_digest,
        "chunks": [
            {"name": name, "sha256": sha256(bundle[name]), "lines": physical_lines(bundle[name].decode())}
            for name in sorted(bundle)
        ],
    }
    bundle["metadata.json"] = (json.dumps(metadata, indent=2, sort_keys=True) + "\n").encode()
    return bundle


def write_bundle(output: Path, bundle: dict[str, bytes]) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=f".{output.name}.stage-", dir=output.parent))
    backup = output.with_name(f".{output.name}.backup-{os.getpid()}")
    try:
        for name, data in bundle.items():
            (stage / name).write_bytes(data)
        if output.exists():
            if not output.is_dir() or backup.exists():
                fail(f"cannot atomically replace {output}")
            output.replace(backup)
        stage.replace(output)
        if backup.exists():
            shutil.rmtree(backup)
    except BaseException:
        if not output.exists() and backup.exists():
            backup.replace(output)
        raise
    finally:
        if stage.exists():
            shutil.rmtree(stage)


def check_bundle(output: Path, bundle: dict[str, bytes]) -> None:
    if not output.is_dir():
        fail(f"generated output is missing: {output}")
    actual = {path.name for path in output.iterdir() if path.is_file()}
    expected = set(bundle)
    if actual != expected:
        fail(f"generated output file set is stale: expected {sorted(expected)}, got {sorted(actual)}")
    for name, data in bundle.items():
        if (output / name).read_bytes() != data:
            fail(f"generated output is stale: {name}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="atomically replace generated output")
    mode.add_argument("--check", action="store_true", help="reject stale generated output")
    parser.add_argument("--spec", type=Path, required=True)
    parser.add_argument("--source-manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        revision, _, _ = load_spec(args.spec)
        verify_manifest(args.source_manifest, revision)
        bundle = expected_bundle(args.spec)
        if args.write:
            write_bundle(args.output, bundle)
            print(f"WROTE: {len(bundle) - 1} chunks; metadata sha256={sha256(bundle['metadata.json'])}")
        else:
            check_bundle(args.output, bundle)
            print(f"PASS: {len(bundle) - 1} chunks; metadata sha256={sha256(bundle['metadata.json'])}")
    except (ChunkError, OSError) as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
