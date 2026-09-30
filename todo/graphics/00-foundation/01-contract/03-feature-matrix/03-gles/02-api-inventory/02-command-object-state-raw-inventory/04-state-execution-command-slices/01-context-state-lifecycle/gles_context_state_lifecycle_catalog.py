"""Finite source-only object-model/shared-lifecycle rules for F03.3.2.2.4.1."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent


def fixed(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        raise ValueError("fixed lifecycle dependency must be a regular file")
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise ValueError("cannot load fixed lifecycle dependency")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


PDF = fixed(HERE.parents[9] / "scripts/graphics/inventories/pdf.py", "lifecycle_pdf")
DATA = fixed(HERE / "gles_context_state_lifecycle_rules.py", "lifecycle_rules")
SCHEMA = fixed(HERE / "gles_context_state_lifecycle_schema.py", "lifecycle_schema")
PROFILE, PAGES, ROUTE = "gles-3.2", 601, "context-state-lifecycle"
PRIMARY_PAGE, PRIMARY_SECTION = 43, "2.6.1.1"
FRAGMENTED = True
ENTRY_FIELDS, DEFAULT_FIELDS = SCHEMA.ENTRY_FIELDS, SCHEMA.DEFAULT_FIELDS
raw_defaults, records, coverage = SCHEMA.raw_defaults, SCHEMA.records, SCHEMA.coverage
RULES, PAGE_HASHES = DATA.RULES, DATA.PAGE_HASHES
RULES_SHA256 = "49619a68bf76a4c3b15a1a07e79f3a684af9ccf6858ab3dc7b2fdf42126632c1"
PAGES_SHA256 = "d44271005b2c028c74ca8a688f48c281f90bb4929938e571f3a5dcec0ac8c339"
BINDING_SHA256 = "f9a4e22a4503434b336e2797688eb37716a7359ba6a447f7afc502d0d40097e3"
WINDOWS_SHA256 = "eb92f0717bd0d73d53c68447ef2c6675a8900aade468c970de1cc6454b65c7bf"
FOOTER = "OpenGL ES 3.2 (May 5, 2022)"
WINDOWS = {
    (43, "2.6.1.1"): ("2.6.1.1 Name Spaces, Name Generation, and Object Creation", FOOTER),
    (44, "2.6.1.1"): ("Generated names do not initially", "2.6.1.2 Name Deletion and Object Deletion"),
    (44, "2.6.1.2"): ("2.6.1.2 Name Deletion and Object Deletion", "2.6.1.3 Shared Object State"),
    (63, "5.1.2"): ("5.1.2 Automatic Unbinding of Deleted Objects", "5.1.3 Deleted Object and Object Name Lifetimes"),
    (63, "5.1.3"): ("5.1.3 Deleted Object and Object Name Lifetimes", FOOTER),
    (65, "5.3"): ("• State-setting commands, such as", "5.3.1 Determining Completion of Changes to an object"),
    (65, "5.3.1"): ("5.3.1 Determining Completion of Changes to an object", "5.3.2 Definitions"),
    (66, "5.3.3"): ("5.3.3 Rules", "Rule 2 While a container object C is bound"),
}


class CatalogError(ValueError):
    """A literal-trigger quotation escaped the finite lifecycle contract."""


def reject(message):
    raise CatalogError(message)


def sha256(value):
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode()).hexdigest()


def page_text(raw, page):
    try:
        return PDF.sealed_pdf_page(raw, page)
    except PDF.PdfError as error:
        reject(str(error))


def source_window(page, section, text):
    if (page, section) not in WINDOWS:
        reject("lifecycle section/page is outside the initial trigger group")
    start, end = WINDOWS[(page, section)]
    if text.count(start) != 1 or text.count(end) != 1:
        reject("lifecycle section heading or ending fence is missing/ambiguous")
    first, last = text.index(start), text.index(end)
    if last <= first:
        reject("lifecycle section fences are reversed")
    return first, text[first:last]


def bound_family(chunk_paths, document):
    rows = [row for chunk in chunk_paths.values() for row in document(chunk).get("families", [])
            if row.get("route") == ROUTE]
    rows.sort(key=lambda row: row.get("source_order", 0))
    if sha256(rows) != BINDING_SHA256:
        reject("lifecycle domain binding omits, reroutes, reorders, or promotes an assigned family")
    binding = [{"id": row["id"], "source_order": row["source_order"]} for row in rows]
    if binding != SCHEMA.BINDING:
        reject("lifecycle family identity or order changed")
    return binding


def facts(raw, pages, binding, normalize):
    if (PROFILE != "gles-3.2" or PAGES != 601 or pages != 601 or PRIMARY_PAGE != 43
            or PRIMARY_SECTION != "2.6.1.1" or binding != SCHEMA.BINDING
            or sha256(RULES) != RULES_SHA256 or sha256(PAGE_HASHES) != PAGES_SHA256
            or sha256(tuple(WINDOWS.items())) != WINDOWS_SHA256):
        reject("lifecycle source contract, quotations, or reviewed page order changed")
    texts = {page: page_text(raw, page) for page, _ in PAGE_HASHES}
    if any(hashlib.sha256(texts[page].encode()).hexdigest() != digest for page, digest in PAGE_HASHES):
        reject("lifecycle reviewed source page is incomplete, changed, or substituted")
    result, last_position = [], (0, 0)
    orders = {item["id"]: item["source_order"] for item in binding}
    for family, page, section, names, parts in RULES:
        expected_family = "object-taxonomy" if section.startswith("2.6.") else "context-and-lifecycle"
        if family != expected_family or family not in orders:
            reject("lifecycle quotation was assigned to an unrelated domain family")
        offset, window = source_window(page, section, texts[page])
        quote = " ".join(parts)
        if window.count(quote) != 1:
            reject("lifecycle quotation is absent, repeated, or outside its exact section")
        position = (page, offset + window.index(quote))
        if position <= last_position:
            reject("lifecycle source quotation order is unstable")
        last_position = position
        if any(not re.search(r"\b" + name + r"\b", quote.replace("- ", "")) for name in names):
            reject("lifecycle trigger is not explicitly present in its quotation")
        try:
            c_names = normalize([("literal", name) for name in names])
        except Exception as error:
            reject(f"declaration grammar rejected a lifecycle literal: {error}")
        if c_names != ["gl" + name for name in names]:
            reject("lifecycle trigger normalization guessed, prefixed, or expanded an API")
        number = len(result) + 1
        result.append([f"gles32-context-state-lifecycle-{number:02d}", family, orders[family],
                       list(names), c_names, list(parts), page, section, number])
    records({"raw_entry_fields": list(ENTRY_FIELDS), "raw_entry_defaults": raw_defaults(binding), "raw_entries": result})
    return result
