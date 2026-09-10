"""Strict inherited JSON and identity primitives for actual Docs staging."""

from __future__ import annotations

import sys
from pathlib import Path

from vulkan_docs_stage_model import StageError, reject

HERE = Path(__file__).resolve().parent
COMPARE = HERE.parent.parent / "03-capture-core-closure/03-compare-fresh-captures"
if str(COMPARE) not in sys.path:
    sys.path.insert(0, str(COMPARE))

from vulkan_docs_compare_parse import canonical as compare_canonical  # noqa: E402
from vulkan_docs_compare_parse import digest as compare_digest  # noqa: E402
from vulkan_docs_compare_parse import document as compare_document  # noqa: E402
from vulkan_docs_compare_parse import exact_object as compare_exact_object  # noqa: E402
from vulkan_docs_compare_parse import identifier as compare_identifier  # noqa: E402
from vulkan_docs_compare_parse import selector as compare_selector  # noqa: E402


def inherited(function, *args):
    try:
        return function(*args)
    except Exception as error:
        reject(str(error))


def canonical(value: object, domain: str) -> str:
    return inherited(compare_canonical, value, domain)


def digest(value: object, label: str) -> str:
    return inherited(compare_digest, value, label)


def document(path: Path) -> dict[str, object]:
    return inherited(compare_document, path)


def exact_object(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    return inherited(compare_exact_object, value, fields, label)


def identifier(value: object, label: str) -> str:
    return inherited(compare_identifier, value, label)


def selector(value: object, label: str) -> str:
    return inherited(compare_selector, value, label)


def fixed_state(value: dict[str, object], label: str) -> None:
    if (value.get("status"), value.get("admitted"), value.get("cutover_ready")) != ("staging-only-unadmitted", False, False):
        reject(f"{label} attempts an admitted or active state")


def bounded(value: object, label: str, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not 0 < value <= maximum:
        reject(f"{label} is outside its explicit bound")
    return value
