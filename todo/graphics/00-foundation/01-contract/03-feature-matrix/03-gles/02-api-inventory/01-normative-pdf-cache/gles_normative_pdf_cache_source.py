#!/usr/bin/env python3
"""Private F03.3.1 authority and external-PDF helpers for F03.3.2.1."""

from __future__ import annotations

import hashlib
import importlib.util
import os
import re
import stat
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
AUTHORITY = HERE.parents[1] / "01-source-authority" / "gles_source_authority.py"
REPO = HERE.parents[7]
PROFILE, PAGES = "gles-3.2", 601
CLASSES = ("command-state", "limit-format")
LAYOUT = "webboxvm-graphics/f02/{record_id}/{sha256}.source"
SOURCE = {"profile": PROFILE, "role": "normative-root", "record_id": "gles-32-spec",
          "record_kind": "upstream-source", "scope": "normative-source",
          "revision": "1cdd228e34966dd6b95bd203e9f84faba0f371a1",
          "sha256": "5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c",
          "bytes": 2198754}
NO_CLAIMS = {name: False for name in ("khronos_selector", "api_support", "conformance",
                                      "certification", "profile_support", "performance")}
STATES = {"mandatory_role_inventory_complete": True, "profile_status": "blocked",
          "blocker": "matrix-incomplete", "api_support": False, "conformance": False,
          "certification": False, "profile_support": False, "performance": False}


class CacheError(ValueError):
    """The external PDF is not safe to treat as the admitted normative root."""


def reject(message: str) -> None:
    raise CacheError(message)


def canonical(value: object) -> bytes:
    import json
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            reject("cache manifest has duplicate JSON fields")
        value[key] = item
    return value


def authority():
    if AUTHORITY.is_symlink() or not AUTHORITY.is_file():
        reject("F03.3.1 authority must be a fixed regular file")
    name, prior = "f03321_gles_source_authority", sys.modules.get("f03321_gles_source_authority")
    spec = importlib.util.spec_from_file_location(name, AUTHORITY)
    if spec is None or spec.loader is None:
        reject("cannot load fixed F03.3.1 authority")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    try:
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != AUTHORITY.resolve():
            reject("F03.3.1 authority resolved from an unexpected path")
        return module
    except CacheError:
        raise
    except Exception as error:
        reject(f"cannot load F03.3.1 authority: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


def admitted_source(locator_class: str, locator: str) -> dict[str, object]:
    if locator_class not in CLASSES:
        reject("only admitted command-state or limit-format classes may read this PDF")
    module = authority()
    try:
        result = module.consume(locator_class, locator)
    except module.BoundaryError as error:
        reject(str(error))
    source = result.get("source") if isinstance(result, dict) else None
    if source != SOURCE:
        reject("F03.3.1 did not return the exact sealed GLES normative root")
    return source


def external_root(value: Path) -> Path:
    if not isinstance(value, Path) or not value.is_absolute() or value.is_symlink():
        reject("cache root must be an absolute nonsymlink path")
    try:
        mode = value.lstat().st_mode
    except OSError as error:
        reject(f"cache root cannot be read: {error}")
    root = value.resolve()
    if not stat.S_ISDIR(mode) or root == REPO or REPO in root.parents:
        reject("cache root must be an external regular directory")
    return root


def cache_file(root: Path, source: dict[str, object]) -> Path:
    if source != SOURCE:
        reject("cache path requires the exact sealed GLES normative source")
    return root / LAYOUT.format(record_id=source["record_id"], sha256=source["sha256"])


def regular_cache_path(root: Path, source: dict[str, object]) -> Path:
    path, current = cache_file(root, source), root
    for part in path.relative_to(root).parts[:-1]:
        current = current / part
        try:
            mode = current.lstat().st_mode
        except OSError as error:
            reject(f"cache layout cannot be read: {error}")
        if stat.S_ISLNK(mode) or not stat.S_ISDIR(mode):
            reject("cache layout must contain only regular directories")
    return path


def identity(value: os.stat_result) -> tuple[int, int, int, int]:
    return value.st_dev, value.st_ino, value.st_size, stat.S_IFMT(value.st_mode)


def pdf_bytes(root: Path, source: dict[str, object]) -> bytes:
    root = external_root(root)
    path, limit = regular_cache_path(root, source), int(source["bytes"])
    nofollow, nonblock = getattr(os, "O_NOFOLLOW", 0), getattr(os, "O_NONBLOCK", 0)
    if not nofollow or not nonblock:
        reject("O_NOFOLLOW and O_NONBLOCK are required for the external PDF cache")
    try:
        before = path.lstat()
        if not stat.S_ISREG(before.st_mode):
            reject("cached PDF must be a regular nonsymlink file")
        if before.st_size != limit:
            reject("cached PDF has a stale, mixed, or oversized byte count")
        descriptor = os.open(path, os.O_RDONLY | nofollow | nonblock)
        with os.fdopen(descriptor, "rb") as stream:
            opened = os.fstat(stream.fileno())
            if not stat.S_ISREG(opened.st_mode) or identity(opened) != identity(before):
                reject("cached PDF changed identity during open")
            raw = stream.read(limit + 1)
        after = path.lstat()
    except OSError as error:
        reject(f"cached PDF cannot be read: {error}")
    if identity(after) != identity(before):
        reject("cached PDF changed identity during read")
    if len(raw) != limit or hashlib.sha256(raw).hexdigest() != source["sha256"]:
        reject("cached PDF does not match the sealed normative identity")
    return raw


def physical_pages(raw: bytes) -> int:
    try:
        result = subprocess.run(["pdfinfo", "-"], input=raw, capture_output=True, check=False)
    except OSError as error:
        reject(f"pdfinfo is required to count physical PDF pages: {error}")
    match = re.search(rb"(?m)^Pages:[ \t]*([1-9][0-9]*)[ \t]*$", result.stdout)
    if result.returncode != 0 or match is None or int(match.group(1)) != PAGES:
        reject("cached PDF has an unexpected physical page count")
    return PAGES


def page_number(locator: str) -> int:
    match = re.search(r":page=([1-9][0-9]*);", locator)
    if match is None:
        reject("authorized locator lacks a physical page")
    return int(match.group(1))
