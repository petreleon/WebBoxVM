#!/usr/bin/env python3
"""Private F03.2.1 authority and external-PDF helpers for F03.2.2.1."""

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
AUTHORITY = HERE.parents[1] / "01-source-authority" / "opengl_source_authority.py"
REPO = HERE.parents[7]
PROFILE, SOURCE_CLASS, PAGES = "opengl-4.6-core", "command-object-state", 851
LAYOUT = "webboxvm-graphics/f02/{record_id}/{sha256}.source"
SOURCE = {"profile": PROFILE, "role": "normative-root", "record_id": "opengl-46-core-spec",
          "record_kind": "upstream-source", "scope": "normative-source",
          "revision": "1cdd228e34966dd6b95bd203e9f84faba0f371a1",
          "sha256": "a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee",
          "bytes": 3003752}
NO_CLAIMS = {key: False for key in ("khronos_selector", "api_support", "conformance", "certification",
                                    "profile_support", "performance")}
STATES = {"mandatory_role_inventory_complete": True, "profile_status": "blocked",
          "blocker": "matrix-incomplete", "api_support": False, "conformance": False,
          "certification": False, "profile_support": False, "performance": False}


class CacheError(ValueError):
    """The external PDF is not safe to treat as the admitted normative root."""


def reject(message: str) -> None:
    raise CacheError(message)


def canonical(value: object) -> bytes:
    import json
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def pairs(items):
    value = {}
    for key, item in items:
        if key in value:
            reject("cache manifest has duplicate JSON fields")
        value[key] = item
    return value


def authority():
    if AUTHORITY.is_symlink() or not AUTHORITY.is_file():
        reject("F03.2.1 authority must be a fixed regular file")
    name, previous = "f03221_opengl_source_authority", sys.modules.get("f03221_opengl_source_authority")
    spec = importlib.util.spec_from_file_location(name, AUTHORITY)
    if spec is None or spec.loader is None:
        reject("cannot load fixed F03.2.1 authority")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    try:
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != AUTHORITY.resolve():
            reject("F03.2.1 authority resolved from an unexpected path")
        return module
    except CacheError:
        raise
    except Exception as error:
        reject(f"cannot load F03.2.1 authority: {error}")
    finally:
        if previous is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = previous


def admitted_source(locator_class: str, locator: str) -> dict[str, object]:
    if locator_class != SOURCE_CLASS:
        reject("only the admitted command-object-state class may read this PDF")
    module = authority()
    try:
        result = module.consume(locator_class, locator)
    except module.BoundaryError as error:
        reject(str(error))
    source = result.get("source") if isinstance(result, dict) else None
    if source != SOURCE:
        reject("F03.2.1 did not return the exact sealed OpenGL normative root")
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
    return root / LAYOUT.format(record_id=source["record_id"], sha256=source["sha256"])


def pdf_bytes(root: Path, source: dict[str, object]) -> bytes:
    path, limit = cache_file(root, source), int(source["bytes"])
    nofollow = getattr(os, "O_NOFOLLOW", 0)
    if not nofollow:
        reject("O_NOFOLLOW is required for the external PDF cache")
    try:
        before = path.lstat()
        if not stat.S_ISREG(before.st_mode) or before.st_size != limit:
            reject("cached PDF has a stale, mixed, or oversized byte count")
        descriptor = os.open(path, os.O_RDONLY | nofollow | getattr(os, "O_NONBLOCK", 0))
        with os.fdopen(descriptor, "rb") as stream:
            if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
                reject("cached PDF is not a regular file")
            raw = stream.read(limit + 1)
    except OSError as error:
        reject(f"cached PDF cannot be read: {error}")
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
