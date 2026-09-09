"""Strict parsers shared by the isolated raw/generated fixture contract."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from pathlib import Path, PurePosixPath
from urllib.parse import urlsplit

from successor_identity_model import (
    GENERATED,
    GENERATED_FIELDS,
    RAW,
    RAW_FIELDS,
    GeneratedMember,
    RawMember,
    reject,
)

HERE = Path(__file__).resolve().parent
SOURCE_MODEL = HERE.parents[5] / "02-fetch-verifier/01-fetch-contract/source_model.py"


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


F02 = reviewed_module("f024_successor_source_model", SOURCE_MODEL)


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            reject("fixture has duplicate JSON object fields")
        value[key] = item
    return value


def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"fixture cannot be read: {error}")
    if not isinstance(value, dict):
        reject("fixture is not a JSON object")
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


def string_list(value: object, label: str) -> tuple[str, ...]:
    if not isinstance(value, list) or not value:
        reject(f"{label} must be a nonempty ordered list")
    return tuple(text(item, label) for item in value)


def strings(value: object, label: str) -> tuple[str, ...]:
    rows = string_list(value, label)
    if len(set(rows)) != len(rows):
        reject(f"{label} has duplicate values")
    return rows


def relative_path(value: object, label: str) -> str:
    value = text(value, label)
    path = PurePosixPath(value)
    if ("\x00" in value or path.is_absolute() or str(path) != value
            or any(part in (".", "..") for part in path.parts)):
        reject(f"{label} is not a safe relative path")
    return value


def file_selector(value: object, label: str) -> str:
    value = relative_path(value, label)
    if value == "." or not PurePosixPath(value).name:
        reject(f"{label} must name a non-root relative file")
    return value


def canonical_digest(value: dict[str, object], field: str, label: str) -> str:
    actual = digest(value.get(field), f"{label} {field}")
    copy = dict(value)
    copy.pop(field, None)
    expected = hashlib.sha256(json.dumps(copy, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    if actual != expected:
        reject(f"{label} has a stale {field}")
    return actual


def canonical(value: object) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def cache(value: object, closure: str, kind: str, identifier: str, digest_value: str, suffix: str) -> str:
    value = relative_path(value, f"{kind} cache")
    expected = PurePosixPath("webboxvm-graphics") / "successor" / closure / kind / identifier
    expected /= f"{digest_value}.{suffix}"
    if value != str(expected):
        reject(f"{kind} cache is outside the successor namespace")
    return value


def byte_count(value: object, label: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not 0 < value <= F02.MAX_INPUT_BYTES:
        reject(f"{label} violates the per-member F02.2 byte limit")
    return value


def raw(value: object, closure: str) -> RawMember:
    if not isinstance(value, dict) or set(value) != RAW_FIELDS or value.get("kind") != RAW:
        reject("raw member has an invalid schema")
    member_id = identifier(value.get("id"), "raw id")
    revision = text(value.get("revision"), "raw revision")
    digest_value = digest(value.get("sha256"), "raw sha256")
    if not F02.COMMIT.fullmatch(revision):
        reject("raw member has an invalid revision")
    url = text(value.get("immutable_url"), "raw URL")
    try:
        F02.immutable_url(url, revision, member_id)
    except F02.ContractError as error:
        reject(f"raw member violates F02.2 URL policy: {error}")
    family = text(value.get("source_family"), "raw source_family")
    license_name = text(value.get("license"), "raw license")
    role = text(value.get("generated_code_role"), "raw generated_code_role")
    provenance = text(value.get("provenance"), "raw provenance")
    selector = file_selector(value.get("selector"), "raw selector")
    if not urlsplit(url).path.endswith("/" + selector):
        reject("raw selector does not name its immutable URL")
    size = byte_count(value.get("bytes"), "raw member")
    path = cache(value.get("local_cache"), closure, "raw", member_id, digest_value, "source")
    return RawMember(member_id, family, url, revision, digest_value, size, license_name,
                     selector, path, role, provenance)


def generated(value: object, closure: str) -> GeneratedMember:
    if not isinstance(value, dict) or set(value) != GENERATED_FIELDS or value.get("kind") != GENERATED:
        reject("generated member has an invalid schema or raw identity")
    member_id = identifier(value.get("id"), "generated id")
    digest_value = digest(value.get("sha256"), "generated sha256")
    producers = strings(value.get("producer_member_ids"), "generated producers")
    selector = file_selector(value.get("selector"), "generated selector")
    generation_id = identifier(value.get("generation_id"), "generated generation_id")
    license_name = text(value.get("license"), "generated license")
    role = text(value.get("generated_code_role"), "generated generated_code_role")
    provenance = text(value.get("provenance"), "generated provenance")
    size = byte_count(value.get("bytes"), "generated member")
    path = cache(value.get("local_cache"), closure, "generated", member_id, digest_value, "derived")
    return GeneratedMember(member_id, producers, generation_id, digest_value, size, license_name,
                           selector, path, role, provenance)
