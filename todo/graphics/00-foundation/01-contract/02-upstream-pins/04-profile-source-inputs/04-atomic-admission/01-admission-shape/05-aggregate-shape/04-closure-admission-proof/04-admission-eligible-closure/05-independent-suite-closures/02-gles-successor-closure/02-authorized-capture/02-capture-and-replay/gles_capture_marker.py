"""Seal an exact GLES cache closure by publishing a self-hashed marker last."""

from __future__ import annotations

import hashlib
import json
import os
import re
import secrets
import stat
from pathlib import Path

from gles_capture_plan import CONTRACT, CapturePlan
from source_model import ExternalCache, reject

MAX_MARKER_BYTES = 32 * 1024
SHA256 = re.compile(r"^[0-9a-f]{64}$")
DIRECTORY_FLAGS = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0)
FILE_FLAGS = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)

def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value:
            reject("GLES capture marker has a duplicate JSON key")
        value[key] = item
    return value

def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "marker_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()

def alike(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right

def relative(current: CapturePlan) -> tuple[str, ...]:
    if not SHA256.fullmatch(current.closure_sha256):
        reject("GLES capture marker has an invalid closure digest")
    return ("webboxvm-graphics", "f02-successor", "gles-cts", current.closure_sha256, "capture.json")

def path(cache: ExternalCache, current: CapturePlan) -> Path:
    return cache.root.joinpath(*relative(current))

def expected(current: CapturePlan) -> dict[str, object]:
    members = [{"id": source.identifier, "role": role, "local_cache": str(source.local_cache),
                "sha256": source.sha256, "bytes": source.byte_count}
               for source, role in zip(current.sources, current.roles)]
    return {"schema": 1, "marker": "gles-successor-capture-v1", "status": "captured-unadmitted",
            "contract_sha256": current.contract_sha256, "closure_sha256": current.closure_sha256,
            "configuration_document_sha256": current.configuration_sha256, "members": members,
            "core_configuration_count": len(current.core_configurations),
            "excluded_configuration_count": len(current.excluded_configurations),
            "producer_execution_proved": False, "output_attestation_present": False,
            "network_during_replay_allowed": False,
            "effects": {name: False for name in CONTRACT.EFFECTS}}

def value(current: CapturePlan) -> dict[str, object]:
    marker = expected(current)
    marker["marker_sha256"] = digest(marker)
    return marker

def opened_directory(name, parent: int | None = None) -> int:
    try:
        descriptor = os.open(name, DIRECTORY_FLAGS) if parent is None else os.open(name, DIRECTORY_FLAGS, dir_fd=parent)
    except OSError as error:
        if isinstance(error, FileNotFoundError):
            raise
        reject(f"GLES capture marker path cannot be safely opened: {error}")
    try:
        if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
            reject("GLES capture marker has a non-directory path component")
        return descriptor
    except Exception:
        os.close(descriptor)
        raise

def directory(cache: ExternalCache, current: CapturePlan, create: bool) -> tuple[int, str]:
    parts = relative(current)
    try:
        descriptor = opened_directory(cache.root)
    except FileNotFoundError:
        reject("GLES capture marker is absent")
    try:
        for part in parts[:-1]:
            try:
                child = opened_directory(part, descriptor)
            except FileNotFoundError:
                if not create:
                    reject("GLES capture marker is absent")
                try:
                    os.mkdir(part, 0o700, dir_fd=descriptor)
                except FileExistsError:
                    pass
                child = opened_directory(part, descriptor)
            os.close(descriptor)
            descriptor = child
        return descriptor, parts[-1]
    except Exception:
        os.close(descriptor)
        raise

def raw(parent: int, name: str) -> bytes:
    try:
        descriptor = os.open(name, FILE_FLAGS, dir_fd=parent)
    except FileNotFoundError:
        reject("GLES capture marker is absent")
    except OSError as error:
        reject(f"GLES capture marker cannot be safely opened: {error}")
    try:
        info = os.fstat(descriptor)
        if not stat.S_ISREG(info.st_mode) or info.st_size > MAX_MARKER_BYTES:
            reject("GLES capture marker is not a bounded regular file")
        chunks, total = [], 0
        while total <= MAX_MARKER_BYTES:
            chunk = os.read(descriptor, min(4096, MAX_MARKER_BYTES + 1 - total))
            if not chunk:
                break
            chunks.append(chunk)
            total += len(chunk)
    finally:
        os.close(descriptor)
    result = b"".join(chunks)
    if len(result) != info.st_size or len(result) > MAX_MARKER_BYTES:
        reject("GLES capture marker changed during bounded read")
    return result

def read(cache: ExternalCache, current: CapturePlan) -> dict[str, object]:
    parent, name = directory(cache, current, False)
    try:
        marker = json.loads(raw(parent, name).decode("utf-8"), object_pairs_hook=pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"GLES capture marker is invalid JSON: {error}")
    finally:
        os.close(parent)
    fields = set(expected(current)) | {"marker_sha256"}
    if not isinstance(marker, dict) or set(marker) != fields or marker.get("marker_sha256") != digest(marker):
        reject("GLES capture marker self-hash or schema is invalid")
    if not alike({key: marker[key] for key in expected(current)}, expected(current)):
        reject("GLES capture marker does not bind this exact closure")
    return marker
def publish(cache: ExternalCache, current: CapturePlan) -> Path:
    parent, name = directory(cache, current, True)
    temporary = f".capture-{secrets.token_hex(16)}.tmp"
    try:
        try:
            descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0), 0o600, dir_fd=parent)
        except FileExistsError:
            reject("GLES capture marker temporary name collided")
        with os.fdopen(descriptor, "wb") as output:
            output.write(json.dumps(value(current), sort_keys=True, separators=(",", ":")).encode())
            output.flush()
            os.fsync(output.fileno())
        try:
            os.link(temporary, name, src_dir_fd=parent, dst_dir_fd=parent, follow_symlinks=False)
        except FileExistsError:
            reject("GLES capture marker already exists")
        except OSError as error:
            reject(f"GLES capture marker cannot be published: {error}")
        os.fsync(parent)
    finally:
        try:
            os.unlink(temporary, dir_fd=parent)
        except FileNotFoundError:
            pass
        os.close(parent)
    return path(cache, current)
