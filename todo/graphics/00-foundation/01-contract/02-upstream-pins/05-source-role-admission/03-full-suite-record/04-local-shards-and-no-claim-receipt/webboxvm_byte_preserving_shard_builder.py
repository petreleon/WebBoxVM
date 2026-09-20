#!/usr/bin/env python3
"""Stream one explicit <=8 MiB local byte-preserving VCTS shard."""

from __future__ import annotations

import argparse
import hashlib
import os
import sys
from pathlib import Path

MODE = "byte-preserving-shard"
MAX_LOCAL_BYTES = 8 * 1024 * 1024
CHUNK = 64 * 1024
API_BYTES = 40296059
API_SHA256 = "a329e8606983b982a34e45f254ac871b3d46b9fce137b78efa15878a5d238a0c"


class BuildError(ValueError):
    """The local shard does not preserve its pinned upstream bytes."""


def reject(message: str) -> None:
    raise BuildError(message)


def digest_file(path: Path) -> tuple[int, str]:
    if path.is_symlink() or not path.is_file():
        reject("input must be a regular non-symlink file")
    total, hasher = 0, hashlib.sha256()
    with path.open("rb") as handle:
        while chunk := handle.read(CHUNK):
            total += len(chunk)
            hasher.update(chunk)
    return total, hasher.hexdigest()


def prepared_destination(path: Path) -> Path:
    if path.exists() or path.is_symlink():
        reject("shard output must be absent and non-symlinked")
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.parent.is_symlink() or not path.parent.is_dir():
        reject("shard output parent must be a regular directory")
    temporary = path.with_name("." + path.name + ".stage")
    if temporary.exists() or temporary.is_symlink():
        reject("shard staging output must be absent")
    return temporary


def write_shard(source: Path, destination: Path, offset: int, length: int,
                source_bytes: int, source_sha256: str, cap: int = MAX_LOCAL_BYTES) -> tuple[int, str]:
    if (type(offset) is not int or type(length) is not int or type(source_bytes) is not int
            or offset < 0 or length < 1 or source_bytes < 1 or length > cap or offset + length > source_bytes):
        reject("shard span is not a bounded source interval")
    if source.is_symlink() or not source.is_file():
        reject("source must be a regular non-symlink file")
    temporary = prepared_destination(destination)
    total, written, source_hash, shard_hash = 0, 0, hashlib.sha256(), hashlib.sha256()
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
    try:
        with source.open("rb") as input_file, os.fdopen(os.open(temporary, flags, 0o600), "wb") as output_file:
            while chunk := input_file.read(CHUNK):
                start, end = total, total + len(chunk)
                source_hash.update(chunk)
                left, right = max(start, offset), min(end, offset + length)
                if left < right:
                    piece = chunk[left - start:right - start]
                    output_file.write(piece)
                    written += len(piece)
                    shard_hash.update(piece)
                total = end
            output_file.flush()
            os.fsync(output_file.fileno())
        if (total, source_hash.hexdigest()) != (source_bytes, source_sha256) or written != length:
            reject("source bytes or requested shard interval changed")
        try:
            os.link(temporary, destination, follow_symlinks=False)
        except FileExistsError:
            reject("shard output appeared during publication")
        os.unlink(temporary)
        return written, shard_hash.hexdigest()
    except BaseException:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass
        raise


def build(source: Path, destination: Path, index: int, count: int, offset: int, length: int) -> tuple[int, str]:
    if type(index) is not int or type(count) is not int or not 0 <= index < count:
        reject("shard index and count are invalid")
    return write_shard(source, destination, offset, length, API_BYTES, API_SHA256)


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", required=True)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--index", type=int, required=True)
    parser.add_argument("--count", type=int, required=True)
    parser.add_argument("--offset", type=int, required=True)
    parser.add_argument("--bytes", dest="length", type=int, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        if args.mode != MODE:
            reject("builder mode must be byte-preserving-shard")
        size, digest = build(args.source, args.output, args.index, args.count, args.offset, args.length)
        print(f"PASS: {args.output} bytes={size} sha256={digest}")
    except BuildError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
