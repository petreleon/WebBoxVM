"""Local prerequisite probes for the profile-independent check runner."""

from __future__ import annotations

import os
import shutil
from pathlib import Path
from typing import Iterable

from catalog import Prerequisite

MODE = {"read": os.R_OK, "write": os.W_OK, "execute": os.X_OK}


def issue(prerequisite: Prerequisite, reason: str) -> dict[str, str]:
    value = {"kind": prerequisite.kind, "value": prerequisite.value, "reason": reason}
    if prerequisite.access:
        value["access"] = prerequisite.access
    return value


def missing_one(root: Path, prerequisite: Prerequisite) -> dict[str, str] | None:
    if prerequisite.kind in ("executable", "browser"):
        if shutil.which(prerequisite.value):
            return None
        return issue(prerequisite, "executable is not available on PATH")
    if prerequisite.kind == "hardware":
        if os.environ.get(prerequisite.value):
            return None
        return issue(prerequisite, "declared hardware environment variable is unset")
    path = root / prerequisite.value
    if prerequisite.kind == "asset":
        if path.exists():
            return None
        return issue(prerequisite, "local asset is missing")
    if not path.exists():
        return issue(prerequisite, "permission probe path is missing")
    if os.access(path, MODE[prerequisite.access or "read"]):
        return None
    return issue(prerequisite, f"{prerequisite.access} permission is unavailable")


def missing(root: Path, prerequisites: Iterable[Prerequisite]) -> list[dict[str, str]]:
    return [result for item in prerequisites if (result := missing_one(root, item)) is not None]
