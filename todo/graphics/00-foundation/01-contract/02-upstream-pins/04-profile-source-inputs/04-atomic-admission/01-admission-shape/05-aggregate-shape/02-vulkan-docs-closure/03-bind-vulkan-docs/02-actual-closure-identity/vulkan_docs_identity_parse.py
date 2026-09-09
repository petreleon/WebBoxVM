"""Strict primitive parsers shared by the actual-Docs successor grammar."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path, PurePosixPath

from vulkan_docs_identity_model import reject

HERE = Path(__file__).resolve().parent
SOURCE_MODEL = HERE.parents[6] / "02-fetch-verifier/01-fetch-contract/source_model.py"


def reviewed_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed module: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(spec.name, None)
        raise
    return module


F02 = reviewed_module("f024_actual_docs_source_model", SOURCE_MODEL)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            reject("Docs identity JSON has duplicate object fields")
        result[key] = value
    return result


def nonfinite(value: str):
    reject(f"Docs identity has a non-finite JSON value: {value}")


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object, parse_constant=nonfinite)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"Docs identity cannot be read: {error}")
    if not isinstance(value, dict):
        reject("Docs identity is not a JSON object")
    return value


def text(value: object, label: str) -> str:
    if not isinstance(value, str) or not value:
        reject(f"{label} must be a nonempty string")
    return value


def digest(value: object, label: str) -> str:
    value = text(value, label)
    if not F02.DIGEST.fullmatch(value) or value == "0" * 64:
        reject(f"{label} must be a nonzero SHA-256")
    return value


def identifier(value: object, label: str) -> str:
    value = text(value, label)
    if not F02.IDENTIFIER.fullmatch(value):
        reject(f"{label} has an invalid id")
    return value


def bounded_int(value: object, label: str, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not 0 < value <= maximum:
        reject(f"{label} is outside its explicit bound")
    return value


def selector(value: object, label: str) -> str:
    value = text(value, label)
    path = PurePosixPath(value)
    if ("\x00" in value or "\\" in value or not value.isascii() or any(ord(char) < 33 or ord(char) > 126 for char in value)
            or path.is_absolute() or str(path) != value or not path.name
            or any(part in (".", "..") for part in path.parts)):
        reject(f"{label} is not a safe relative file selector")
    return value


def strings(value: object, label: str) -> tuple[str, ...]:
    if not isinstance(value, list) or not value:
        reject(f"{label} must be a nonempty ordered list")
    rows = tuple(text(item, label) for item in value)
    if len(set(rows)) != len(rows):
        reject(f"{label} has duplicate values")
    return rows


def ordered(value: object, label: str) -> tuple[str, ...]:
    if not isinstance(value, list) or not value:
        reject(f"{label} must be a nonempty ordered list")
    return tuple(text(item, label) for item in value)


def canonical(value: object, domain: str = "vulkan-docs-identity-v1") -> str:
    if not isinstance(domain, str) or not domain.isascii() or not domain:
        reject("Docs identity canonicalization has an invalid domain")
    try:
        rendered = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False)
    except (TypeError, ValueError) as error:
        reject(f"Docs identity cannot be canonicalized: {error}")
    return hashlib.sha256(domain.encode("ascii") + b"\0" + rendered.encode("utf-8")).hexdigest()


def self_digest(value: dict[str, object], field: str, label: str) -> str:
    actual = digest(value.get(field), f"{label} {field}")
    rendered = dict(value)
    rendered.pop(field, None)
    if actual != canonical(rendered, f"webboxvm-graphics-{field}-v1"):
        reject(f"{label} has a stale {field}")
    return actual
