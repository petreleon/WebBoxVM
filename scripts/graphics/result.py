"""Stable local JSON output and artifact hashing for graphics checks."""

from __future__ import annotations

import hashlib
import json
import os
import tempfile
from pathlib import Path
from typing import Any


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def artifact(root: Path, relative: str) -> dict[str, Any]:
    path = root / relative
    try:
        data = path.read_bytes()
    except OSError as error:
        return {"path": relative, "bytes": None, "sha256": None, "error": str(error)}
    return {"path": relative, "bytes": len(data), "sha256": sha256(data)}


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    data = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    with tempfile.NamedTemporaryFile(prefix=f".{path.name}.", dir=path.parent, delete=False) as output:
        output.write(data)
        temporary = Path(output.name)
    try:
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)
