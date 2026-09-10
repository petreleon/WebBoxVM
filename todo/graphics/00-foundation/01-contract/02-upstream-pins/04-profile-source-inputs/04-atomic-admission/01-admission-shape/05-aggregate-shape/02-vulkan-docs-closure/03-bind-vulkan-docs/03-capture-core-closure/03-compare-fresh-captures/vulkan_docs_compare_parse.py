"""Inherited fail-closed primitives for the independent Docs capture comparison."""

from __future__ import annotations

import sys
from pathlib import Path

from vulkan_docs_compare_model import reject

HERE = Path(__file__).resolve().parent
SCOPE = HERE.parent / "02-bind-core-input-scope"
if str(SCOPE) not in sys.path:
    sys.path.insert(0, str(SCOPE))

from vulkan_docs_scope_parse import canonical as scope_canonical  # noqa: E402
from vulkan_docs_scope_parse import digest as scope_digest  # noqa: E402
from vulkan_docs_scope_parse import document as scope_document  # noqa: E402
from vulkan_docs_scope_parse import exact_object as scope_exact_object  # noqa: E402
from vulkan_docs_scope_parse import identifier as scope_identifier  # noqa: E402
from vulkan_docs_scope_parse import selector as scope_selector  # noqa: E402


def inherited(function, *args):
    try:
        return function(*args)
    except Exception as error:
        reject(str(error))


def canonical(value: object, domain: str) -> str:
    return inherited(scope_canonical, value, domain)


def digest(value: object, label: str) -> str:
    return inherited(scope_digest, value, label)


def document(path: Path) -> dict[str, object]:
    return inherited(scope_document, path)


def exact_object(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    return inherited(scope_exact_object, value, fields, label)


def identifier(value: object, label: str) -> str:
    return inherited(scope_identifier, value, label)


def selector(value: object, label: str) -> str:
    return inherited(scope_selector, value, label)
