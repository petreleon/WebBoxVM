"""Fresh fixture validation and fixed WebBoxVM-repository anchoring."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

from successor_cache_model import reject

HERE = Path(__file__).resolve().parent
IDENTITY_DIR = HERE.parent / "01-successor-identity"


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


def project_root() -> Path:
    for candidate in HERE.parents:
        if (candidate / ".git").exists():
            return candidate.resolve()
    raise RuntimeError("cannot locate the WebBoxVM repository")


IDENTITY = module("f024_successor_cache_identity", IDENTITY_DIR / "successor_identity_contract.py")
PROJECT_ROOT = project_root()


def trusted_repository(value: Path) -> Path:
    if not isinstance(value, Path) or value.is_symlink() or value.resolve(strict=False) != PROJECT_ROOT:
        reject("successor cache repository must be the real project root")
    return PROJECT_ROOT


def plan(fixture_path: Path, predecessor_paths: object):
    try:
        result = IDENTITY.validate(fixture_path, predecessor_paths)
    except IDENTITY.IdentityError as error:
        reject(f"successor fixture validation failed: {error}")
    if result.admitted or result.cutover_ready:
        reject("successor fixture attempts admission")
    return result.cache_plan
