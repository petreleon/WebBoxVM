"""Strict inherited primitives for the core input/scope binder."""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path

from vulkan_docs_scope_model import MAX_DOCUMENT_BYTES, ScopeError, reject

HERE = Path(__file__).resolve().parent
OBSERVER = HERE.parent / "01-observe-pinned-build-inputs"
if str(OBSERVER) not in sys.path:
    sys.path.insert(0, str(OBSERVER))

from vulkan_docs_observer_parse import canonical as observer_canonical  # noqa: E402
from vulkan_docs_observer_parse import digest as observer_digest  # noqa: E402
from vulkan_docs_observer_parse import document as observer_document  # noqa: E402
from vulkan_docs_observer_parse import identifier as observer_identifier  # noqa: E402
from vulkan_docs_observer_parse import phases as observer_phases  # noqa: E402
from vulkan_docs_observer_parse import positive as observer_positive  # noqa: E402
from vulkan_docs_observer_parse import selector as observer_selector  # noqa: E402
from vulkan_docs_observer_parse import source_limit as observer_source_limit  # noqa: E402


def inherited(function, *args):
    try:
        return function(*args)
    except Exception as error:
        reject(str(error))


def canonical(value: object, domain: str) -> str:
    return inherited(observer_canonical, value, domain)


def digest(value: object, label: str) -> str:
    return inherited(observer_digest, value, label)


def document(path: Path) -> dict[str, object]:
    try:
        if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_DOCUMENT_BYTES:
            reject("scope document is not a bounded regular file")
    except OSError as error:
        reject(f"scope document cannot be read: {error}")
    return inherited(observer_document, path)


def identifier(value: object, label: str) -> str:
    return inherited(observer_identifier, value, label)


def selector(value: object, label: str) -> str:
    return inherited(observer_selector, value, label)


def phases(value: object, label: str) -> tuple[str, ...]:
    return inherited(observer_phases, value, label)


def positive(value: object, label: str, maximum: int) -> int:
    return inherited(observer_positive, value, label, maximum)


def source_limit(value: object, label: str) -> int:
    return inherited(observer_source_limit, value, label)


def nonnegative(value: object, label: str, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not 0 <= value <= maximum:
        reject(f"{label} is outside its explicit bound")
    return value


def file_digest(path: Path, label: str, maximum: int) -> str:
    try:
        if path.is_symlink() or not path.is_file() or path.stat().st_size > maximum:
            reject(f"{label} is not a regular file")
        result = hashlib.sha256()
        with path.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                result.update(chunk)
        return result.hexdigest()
    except OSError as error:
        reject(f"{label} cannot be read: {error}")


def exact_object(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject(f"{label} has an invalid schema")
    return value


def require_safe_child(root: Path, relative: str, label: str) -> Path:
    name = selector(relative, label)
    try:
        root = root.resolve(strict=True)
        path = root
        for part in name.split("/"):
            path /= part
            if path.is_symlink():
                reject(f"{label} contains a symlink")
        resolved = path.resolve(strict=True)
        resolved.relative_to(root)
    except (OSError, ValueError) as error:
        reject(f"{label} escapes its artifact root: {error}")
    if path.is_symlink() or not resolved.is_file():
        reject(f"{label} is not a regular artifact file")
    return resolved
