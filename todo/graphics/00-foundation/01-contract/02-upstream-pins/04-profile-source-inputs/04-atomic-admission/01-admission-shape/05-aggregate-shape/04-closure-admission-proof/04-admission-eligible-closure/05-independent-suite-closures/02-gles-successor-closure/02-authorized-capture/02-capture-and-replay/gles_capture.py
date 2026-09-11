#!/usr/bin/env python3
"""Capture or replay the sealed, unadmitted six-member GLES source closure."""

from __future__ import annotations

import argparse
import os
import stat
import sys
from pathlib import Path

from gles_capture_marker import path as marker_path, publish, read
from gles_capture_plan import CapturePlan, plan, validate_root
from source_cache import fetch_to_cache, verify_payload
from source_model import ContractError, ExternalCache, SourceInput, reject, repository_root

CHUNK_SIZE = 64 * 1024
HERE = Path(__file__).resolve().parent
DIRECTORY_FLAGS = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
FILE_FLAGS = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)


def parent(cache: ExternalCache, source: SourceInput) -> tuple[int, str]:
    try:
        descriptor = os.open(cache.root, DIRECTORY_FLAGS)
    except OSError as error:
        reject(f"GLES capture cannot safely open cache root: {error}")
    try:
        for part in source.local_cache.parts[:-1]:
            child = os.open(part, DIRECTORY_FLAGS, dir_fd=descriptor)
            if not stat.S_ISDIR(os.fstat(child).st_mode):
                os.close(child)
                reject(f"GLES capture path for {source.identifier} is not a directory")
            os.close(descriptor)
            descriptor = child
        return descriptor, source.local_cache.name
    except FileNotFoundError:
        os.close(descriptor)
        reject(f"GLES capture is missing {source.identifier}")
    except OSError as error:
        os.close(descriptor)
        reject(f"GLES capture path for {source.identifier} cannot be safely opened: {error}")
    except Exception:
        os.close(descriptor)
        raise


def payload(cache: ExternalCache, source: SourceInput) -> bytes:
    directory, name = parent(cache, source)
    try:
        descriptor = os.open(name, FILE_FLAGS, dir_fd=directory)
    except FileNotFoundError:
        os.close(directory)
        reject(f"GLES capture is missing {source.identifier}")
    except OSError as error:
        os.close(directory)
        reject(f"GLES capture cannot safely open {source.identifier}: {error}")
    try:
        info, chunks, total = os.fstat(descriptor), [], 0
        if not stat.S_ISREG(info.st_mode) or info.st_size != source.byte_count:
            reject(f"GLES capture member {source.identifier} is not an exact regular file")
        while total <= source.byte_count:
            chunk = os.read(descriptor, min(CHUNK_SIZE, source.byte_count + 1 - total))
            if not chunk: break
            chunks.append(chunk)
            total += len(chunk)
    finally:
        os.close(descriptor)
        os.close(directory)
    value = b"".join(chunks)
    verify_payload(source, value)
    return value


def closure(cache: ExternalCache, current: CapturePlan) -> dict[str, bytes]:
    values = {source.identifier: payload(cache, source) for source in current.sources}
    if len(values) != 6:
        reject("GLES capture has duplicate or partial members")
    validate_root(values[current.root.identifier], current)
    return values


def fresh(cache: ExternalCache) -> None:
    if cache.root.exists() or cache.root.is_symlink():
        reject("GLES capture requires a fresh dedicated external cache root")


def capture(cache: ExternalCache, timeout: float = 30.0, opener=None, current: CapturePlan | None = None) -> Path:
    current = current or plan()
    if timeout <= 0:
        reject("GLES capture timeout must be positive")
    fresh(cache)
    for source in current.sources:
        fetch_to_cache(cache, source, timeout, opener)
    closure(cache, current)
    return publish(cache, current)


def replay(cache: ExternalCache, current: CapturePlan | None = None) -> Path:
    current = current or plan()
    read(cache, current)
    closure(cache, current)
    return marker_path(cache, current)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--capture", action="store_true")
    action.add_argument("--replay", action="store_true")
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=30.0)
    args = parser.parse_args()
    try:
        cache = ExternalCache.from_path(args.cache_root, repository_root(HERE))
        target = capture(cache, args.timeout) if args.capture else replay(cache)
    except ContractError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"PASS: {'capture' if args.capture else 'replay'} captured-unadmitted {target}")


if __name__ == "__main__":
    main()
