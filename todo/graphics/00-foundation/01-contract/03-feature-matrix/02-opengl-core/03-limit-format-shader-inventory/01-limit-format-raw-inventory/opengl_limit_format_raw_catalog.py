#!/usr/bin/env python3
"""Fixed raw table anchors and private source loading for F03.2.3.1."""

from __future__ import annotations

import importlib.util
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = HERE.parents[1] / "02-command-object-state-inventory/01-normative-pdf-cache/opengl_normative_pdf_cache.py"
PROFILE, LIMIT_CLASS = "opengl-4.6-core", "limit-format"
LOCATOR = "opengl46-core-pdf-v1:page=657;section=23.53"
SECTION = re.compile(r"^[1-9][0-9]*(?:\.[1-9][0-9]*)*$")
ROWS = (
    ("23.53", 657, 2, "MAX CLIP DISTANCES", "limit", "13.7"), ("23.53", 657, 3, "MAX CULL DISTANCES", "limit", "13.7"),
    ("23.53", 657, 4, "MAX COMBINED CLIP AND CULL DISTANCES", "limit", "13.7"), ("23.53", 657, 5, "SUBPIXEL BITS", "limit", "14"),
    ("23.53", 657, 6, "MAX ELEMENT INDEX", "limit", "10.4"), ("23.53", 657, 8, "MAX 3D TEXTURE SIZE", "limit", "8.5"),
    ("23.53", 657, 9, "MAX TEXTURE SIZE", "limit", "8.5"), ("23.53", 657, 10, "MAX ARRAY TEXTURE LAYERS", "limit", "8.5"),
    ("23.53", 657, 11, "MAX TEXTURE LOD BIAS", "limit", "8.14"), ("23.53", 657, 12, "MAX CUBE MAP TEXTURE SIZE", "limit", "8.5"),
    ("23.53", 657, 13, "MAX RENDERBUFFER SIZE", "limit", "9.2.4"), ("23.54", 658, 1, "MAX VIEWPORT DIMS", "limit", "13.8.1"),
    ("23.54", 658, 2, "MAX VIEWPORTS", "limit", "13.8.1"), ("23.54", 658, 3, "VIEWPORT SUBPIXEL BITS", "limit", "13.8.1"),
    ("23.54", 658, 4, "VIEWPORT BOUNDS RANGE", "limit", "13.8.1"), ("23.54", 658, 7, "POINT SIZE RANGE", "limit", "14.4"),
    ("23.54", 658, 8, "POINT SIZE GRANULARITY", "limit", "14.4"), ("23.54", 658, 9, "ALIASED LINE WIDTH RANGE", "limit", "14.5"),
    ("23.54", 658, 10, "SMOOTH LINE WIDTH RANGE", "limit", "14.5"), ("23.54", 658, 11, "SMOOTH LINE WIDTH GRANULARITY", "limit", "14.5"),
    ("23.54", 658, 12, "MAX ELEMENTS INDICES", "limit", "10.3"), ("23.54", 658, 13, "MAX ELEMENTS VERTICES", "limit", "10.3"),
    ("23.55", 659, 1, "MAX VERTEX ATTRIB RELATIVE OFFSET", "limit", "10.3"), ("23.55", 659, 2, "MAX VERTEX ATTRIB BINDINGS", "limit", "10.3"),
    ("23.55", 659, 3, "MAX VERTEX ATTRIB STRIDE", "limit", "10.3"), ("23.55", 659, 4, "NUM COMPRESSED TEXTURE FORMATS", "format-property", "8.7"),
    ("23.55", 659, 5, "COMPRESSED TEXTURE FORMATS", "format-property", "8.7"), ("23.55", 659, 6, "MAX TEXTURE BUFFER SIZE", "limit", "8.9"),
    ("23.55", 659, 7, "MAX RECTANGLE TEXTURE SIZE", "limit", "8.5"), ("23.55", 659, 13, "MIN MAP BUFFER ALIGNMENT", "limit", "6.3"),
    ("23.55", 659, 14, "TEXTURE BUFFER OFFSET ALIGNMENT", "limit", "8.9"), ("23.71", 675, 1, "SAMPLES", "format-property", "22.3"),
    ("23.71", 675, 2, "NUM SAMPLE COUNTS", "format-property", "22.3"),
)
EXCLUDED = (
    ("23.53", 1, "CONTEXT RELEASE BEHAVIOR", "22.2", "non-limit-format-behavior"),
    ("23.53", 7, "PRIMITIVE RESTART FOR PATCHES SUPPORTED", "10.3.6", "boolean-capability-not-limit-format"),
    ("23.53", 14, "MAX TEXTURE MAX ANISOTROPY", "8.14", "extension-class-unadmitted"),
    ("23.54", 5, "LAYER PROVOKING VERTEX", "11.3.4", "state-semantics-not-limit-format"),
    ("23.54", 6, "VIEWPORT INDEX PROVOKING VERTEX", "11.3.4", "state-semantics-not-limit-format"),
    ("23.55", 8, "NUM PROGRAM BINARY FORMATS", "7.5", "shader-program-semantics-unadmitted"),
    ("23.55", 9, "PROGRAM BINARY FORMATS", "7.5", "shader-program-semantics-unadmitted"),
    ("23.55", 10, "NUM SHADER BINARY FORMATS", "7.2", "shader-program-semantics-unadmitted"),
    ("23.55", 11, "SHADER BINARY FORMATS", "7.2", "shader-program-semantics-unadmitted"),
    ("23.55", 12, "SHADER COMPILER", "7", "shader-program-semantics-unadmitted"),
)
UNCOVERED = ("tables-23.56-through-23.70-version-extension-or-shader-families-unclassified",
             "chapter-local-limit-and-format-tables-outside-reviewed-table-scope-unclassified")
TABLE_PAGES = {"23.53": 657, "23.54": 658, "23.55": 659, "23.71": 675}
UNADMITTED_INPUTS = ("shader", "extension", "gl.xml", "desktop-glsl", "lower-opengl-profile")


class InventoryError(ValueError):
    """A raw limit/format source fact escapes its sealed boundary."""


def reject(message: str) -> None:
    raise InventoryError(message)


def cache_api():
    if CACHE.is_symlink() or not CACHE.is_file():
        reject("fixed F03.2.2.1 cache boundary is unavailable")
    name, prior = "f03231_opengl_pdf_cache", sys.modules.get("f03231_opengl_pdf_cache")
    try:
        spec = importlib.util.spec_from_file_location(name, CACHE)
        if spec is None or spec.loader is None:
            reject("cannot load fixed F03.2.2.1 cache boundary")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != CACHE.resolve():
            reject("F03.2.2.1 cache boundary resolved from an unexpected path")
        return module
    except InventoryError:
        raise
    except Exception as error:
        reject(f"cannot load fixed F03.2.2.1 cache boundary: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior


def source_input(cache_root: Path, locator_class: str = LIMIT_CLASS, locator: str = LOCATOR) -> dict[str, object]:
    if locator_class != LIMIT_CLASS or locator != LOCATOR:
        reject("only the fixed admitted limit-format PDF input is allowed")
    cache, manifest = cache_api(), None
    try:
        manifest = cache.manifest()
        authority = cache.SOURCE_API.authority()
        boundary, consumed = authority.validate(), authority.consume(locator_class, locator)
    except Exception as error:
        reject(str(error))
    if not isinstance(consumed, dict) or not isinstance(boundary, dict):
        reject("F03.2.1 did not return a source decision")
    source, decision = consumed.get("source"), consumed.get("decision")
    if (not isinstance(source, dict) or source != cache.SOURCE or not isinstance(decision, dict)
            or decision.get("id") != LIMIT_CLASS or decision.get("availability") != "available"
            or boundary.get("normative_root") != source or boundary.get("profile") != PROFILE):
        reject("F03.2.1 did not admit the exact limit-format source class")
    try:
        root = cache.external_root(cache_root)
        raw = cache.pdf_bytes(root, source)
        pages = cache.physical_pages(raw)
    except Exception as error:
        reject(str(error))
    if pages != cache.PAGES or manifest.get("source") != source:
        reject("external PDF cache provenance differs from F03.2.2.1")
    return {"source": source, "decision": decision, "cache_boundary_sha256": manifest["cache_boundary_sha256"],
            "cache_layout": manifest["cache_layout"], "physical_pdf_pages": pages,
            "source_authority_boundary_sha256": boundary["boundary_sha256"],
            "source_contract_sha256": boundary["source_contract_sha256"],
            "inventory_lock_sha256": boundary["inventory_lock_sha256"], "cache_root": str(root), "raw": raw}


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-layout", "-", "-"],
                                input=raw, capture_output=True, check=False)
        text = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed PDF page: {error}")
    if result.returncode != 0:
        reject("cannot read sealed PDF page")
    return text


def anchored_pages(raw: bytes, pages: int) -> dict[int, str]:
    texts = {page: page_text(raw, page) for page in TABLE_PAGES.values()}
    all_rows = [(table, row, name) for table, _, row, name, *_ in ROWS]
    all_rows.extend((table, row, name) for table, row, name, *_ in EXCLUDED)
    for table, page in TABLE_PAGES.items():
        if not 1 <= page <= pages or texts[page].count(f"Table {table}:") != 1:
            reject("sealed PDF has a missing or ambiguous table anchor")
        prior = -1
        for _, _, name in sorted((item for item in all_rows if item[0] == table), key=lambda item: item[1]):
            matches = list(re.finditer(rf"(?m)^[ \t]*{re.escape(name)}(?=[ \t]|$)", texts[page]))
            if len(matches) != 1 or matches[0].start() <= prior:
                reject("sealed PDF has a missing, ambiguous, or reordered table row anchor")
            prior = matches[0].start()
    return texts


def facts(raw: bytes, pages: int) -> list[dict[str, object]]:
    texts, result = anchored_pages(raw, pages), []
    for order, (table, page, row, name, kind, section) in enumerate(ROWS, 1):
        if (TABLE_PAGES.get(table) != page or not SECTION.fullmatch(section) or not (1 <= page <= pages)
                or kind not in ("limit", "format-property") or name not in texts[page]):
            reject("catalog has an invalid source anchor")
        result.append({"raw_id": f"opengl46-table-{table.replace('.', '-')}-row-{row:02}", "name": name,
                       "kind": kind, "physical_page": page, "numeric_section": section,
                       "source_locator": f"opengl46-core-pdf-v1:page={page};section={section}", "table": table,
                       "table_row": row, "source_order": order, "derivation_class": LIMIT_CLASS})
    return result


def coverage() -> dict[str, object]:
    tables = []
    for table, page in TABLE_PAGES.items():
        tables.append({"table": table, "physical_page": page,
                       "included_rows": [row for current, _, row, *_ in ROWS if current == table],
                       "excluded_rows": [{"row": row, "name": name, "numeric_section": section, "reason": reason}
                                         for current, row, name, section, reason in EXCLUDED if current == table]})
    return {"manifest_id": "opengl46-reviewed-limit-format-tables-v1", "complete": False,
            "coverage_decision": "bounded-reviewed-table-rows-only", "reviewed_tables": tables,
            "unadmitted_inputs": list(UNADMITTED_INPUTS), "unreviewed_families": list(UNCOVERED)}
