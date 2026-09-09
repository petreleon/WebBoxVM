"""Strict parsing helpers for the Vulkan Docs observer artifacts."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

from vulkan_docs_observer_model import MAX_EVENTS, reject

MAX_DOCUMENT_BYTES = 1024 * 1024
MAX_JSONL_BYTES = 64 * 1024 * 1024
MAX_JSONL_LINE_BYTES = 128 * 1024

HERE = Path(__file__).resolve().parent
IDENTITY = HERE.parent.parent / "02-actual-closure-identity"
if str(IDENTITY) not in sys.path:
    sys.path.insert(0, str(IDENTITY))

from vulkan_docs_identity_parse import F02, canonical as identity_canonical, digest as identity_digest  # noqa: E402
from vulkan_docs_identity_parse import identifier as identity_identifier, selector as identity_selector, text as identity_text


def inherited(function, *args):
    try:
        return function(*args)
    except Exception as error:
        reject(str(error))


def canonical(value: object, domain: str) -> str:
    return inherited(identity_canonical, value, domain)


def digest(value: object, label: str) -> str:
    return inherited(identity_digest, value, label)


def identifier(value: object, label: str) -> str:
    return inherited(identity_identifier, value, label)


def selector(value: object, label: str) -> str:
    return inherited(identity_selector, value, label)


def text(value: object, label: str) -> str:
    return inherited(identity_text, value, label)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            reject("observer JSON has duplicate object fields")
        result[key] = value
    return result


def nonfinite(value: str):
    reject(f"observer JSON has a non-finite value: {value}")


def json_value(data: str, label: str):
    try:
        return json.loads(data, object_pairs_hook=unique_object, parse_constant=nonfinite)
    except json.JSONDecodeError as error:
        reject(f"{label} is not valid JSON: {error}")


def document(path: Path) -> dict[str, object]:
    try:
        data = path.read_bytes()
    except (OSError, UnicodeDecodeError) as error:
        reject(f"observer document cannot be read: {error}")
    if len(data) > MAX_DOCUMENT_BYTES:
        reject("observer document exceeds its explicit byte bound")
    try:
        value = json_value(data.decode("utf-8"), "observer document")
    except UnicodeDecodeError as error:
        reject(f"observer document cannot be read: {error}")
    if not isinstance(value, dict):
        reject("observer document is not an object")
    return value


def jsonl(path: Path) -> tuple[dict[str, object], ...]:
    try:
        data = path.read_bytes()
    except OSError as error:
        reject(f"observer JSONL cannot be read: {error}")
    if len(data) > MAX_JSONL_BYTES:
        reject("observer JSONL exceeds its explicit byte bound")
    lines = data.splitlines()
    if not lines or len(lines) > MAX_EVENTS:
        reject("observer JSONL has an invalid event count")
    rows = []
    for number, line in enumerate(lines, 1):
        if len(line) > MAX_JSONL_LINE_BYTES:
            reject("observer JSONL line exceeds its explicit byte bound")
        try:
            value = json_value(line.decode("utf-8"), f"observer JSONL line {number}")
        except UnicodeDecodeError as error:
            reject(f"observer JSONL cannot be read: {error}")
        if not isinstance(value, dict):
            reject(f"observer JSONL line {number} is not an object")
        rows.append(value)
    return tuple(rows)


def digest_file(path: Path, label: str) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        reject(f"{label} cannot be read: {error}")


def positive(value: object, label: str, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not 0 < value <= maximum:
        reject(f"{label} is outside its explicit bound")
    return value


def phases(value: object, label: str) -> tuple[str, ...]:
    if not isinstance(value, list) or not value:
        reject(f"{label} must be a nonempty ordered list")
    result = tuple(text(item, label) for item in value)
    if result != tuple(sorted(set(result))):
        reject(f"{label} must be sorted and unique")
    return result


def hex_bytes(value: object, label: str) -> bytes:
    value = text(value, label)
    if len(value) % 2 or any(character not in "0123456789abcdef" for character in value):
        reject(f"{label} is not lowercase hexadecimal")
    try:
        return bytes.fromhex(value)
    except ValueError as error:
        reject(f"{label} cannot be decoded: {error}")


def path_selector(value: str, prefix: str, label: str) -> str | None:
    if not value.startswith(prefix):
        return None
    result = value[len(prefix):]
    return selector(result, label)


def source_limit(value: object, label: str) -> int:
    return positive(value, label, F02.MAX_INPUT_BYTES)
