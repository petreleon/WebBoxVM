#!/usr/bin/env python3
"""Private sealed-PDF catalog for the bounded F03.3.2.3 raw inventory."""

from __future__ import annotations

import importlib.util
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = HERE.parent / "01-normative-pdf-cache/gles_normative_pdf_cache.py"
PROFILE, LIMIT_CLASS, LOCATOR, PAGES = "gles-3.2", "limit-format", "gles32-pdf-v1:page=504;section=21", 601
SOURCE = {"bytes": 2198754, "profile": PROFILE, "record_id": "gles-32-spec", "record_kind": "upstream-source",
          "revision": "1cdd228e34966dd6b95bd203e9f84faba0f371a1", "role": "normative-root",
          "scope": "normative-source", "sha256": "5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c"}
# route | kind or exclusion reason | numeric section | exact PDF text anchor
TABLES = (
    ("21.40", 504, "Table 21.40: Implementation Dependent Values", """
admit|limit|13|SUBPIXEL BITS
admit|limit|10.5|MAX ELEMENT INDEX
admit|limit|8.5|MAX 3D TEXTURE SIZE
admit|limit|8.5|MAX TEXTURE SIZE
admit|limit|8.5|MAX ARRAY TEXTURE LAYERS
admit|limit|8.14|MAX TEXTURE LOD BIAS
admit|limit|8.5|MAX CUBE MAP TEXTURE SIZE
admit|limit|9.2.4|MAX RENDERBUFFER SIZE
admit|limit|13.5|ALIASED POINT SIZE RANGE
admit|limit|13.6|ALIASED LINE WIDTH RANGE
admit|limit|13.6.4|MULTISAMPLE LINE WIDTH RANGE
admit|limit|13.6.4|MULTISAMPLE LINE WIDTH GRANULARITY
admit|limit|15.2.1|MAX DRAW BUFFERS
admit|limit|9.2|MAX FRAMEBUFFER WIDTH
admit|limit|9.2|MAX FRAMEBUFFER HEIGHT
admit|limit|9.2.1|MAX FRAMEBUFFER LAYERS
admit|format-property|9.2|MAX FRAMEBUFFER SAMPLES
admit|limit|9.2.7|MAX COLOR ATTACHMENTS"""),
    ("21.41", 505, "Table 21.41: Implementation Dependent Values (cont.)", """
shader|shader-semantics-unadmitted|14.1|MIN FRAGMENT INTERPOLATION OFFSET
shader|shader-semantics-unadmitted|14.1|MAX FRAGMENT INTERPOLATION OFFSET
precision|precision-semantics-unadmitted|14.1|FRAGMENT INTERPOLATION OFFSET BITS
admit|limit|12.6.1|MAX VIEWPORT DIMS
admit|format-property|9.2.4|MAX SAMPLES
admit|limit|13.8.3|MAX SAMPLE MASK WORDS
admit|format-property|13.8.3|MAX COLOR TEXTURE SAMPLES
admit|format-property|13.8.3|MAX DEPTH TEXTURE SAMPLES
admit|format-property|9.2.4|MAX INTEGER SAMPLES
admit|limit|4.1.1|MAX SERVER WAIT TIMEOUT
command-state|state-semantics-not-limit-format|11.3.4|LAYER PROVOKING VERTEX
command-state|boolean-capability-not-limit-format|10.3.4|PRIMITIVE RESTART FOR PATCHES SUPPORTED"""),
    ("21.42", 506, "Table 21.42: Implementation Dependent Values (cont.)", """
admit|limit|10.3|MAX VERTEX ATTRIB RELATIVE OFFSET
admit|limit|10.3|MAX VERTEX ATTRIB BINDINGS
admit|limit|10.3|MAX VERTEX ATTRIB STRIDE
admit|limit|10.3|MAX ELEMENTS INDICES
admit|limit|10.3|MAX ELEMENTS VERTICES
admit|limit|8.9|MAX TEXTURE BUFFER SIZE
admit|format-property|8.7|NUM COMPRESSED TEXTURE FORMATS
admit|format-property|8.7|COMPRESSED TEXTURE FORMATS
shader|shader-program-semantics-unadmitted|7.5|NUM PROGRAM BINARY FORMATS
shader|shader-program-semantics-unadmitted|7.5|PROGRAM BINARY FORMATS
shader|shader-program-semantics-unadmitted|7.2|NUM SHADER BINARY FORMATS
shader|shader-program-semantics-unadmitted|7.2|SHADER BINARY FORMATS
shader|shader-program-semantics-unadmitted|11.1|SHADER COMPILER
admit|limit|8.9|TEXTURE BUFFER OFFSET ALIGNMENT
precision|precision-semantics-unadmitted|7.12|Shader data type ranges
precision|precision-semantics-unadmitted|7.12|Shader data type precisions"""),
)
class InventoryError(ValueError):
    """The sealed raw-input boundary cannot support this catalog."""
def reject(message: str) -> None:
    raise InventoryError(message)
def exact(left: object, right: object) -> bool:
    if type(left) is not type(right): return False
    if isinstance(left, dict): return set(left) == set(right) and all(exact(left[key], right[key]) for key in left)
    if isinstance(left, list): return len(left) == len(right) and all(exact(a, b) for a, b in zip(left, right))
    return left == right
def private(path: Path, name: str):
    if path.is_symlink() or not path.is_file():
        reject("fixed normative-PDF cache must be a regular file")
    prior = sys.modules.get(name)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            reject("cannot load fixed normative-PDF cache")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            reject("normative-PDF cache resolved from an unexpected path")
        return module
    except InventoryError:
        raise
    except Exception as error:
        reject(f"cannot load fixed normative-PDF cache: {error}")
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior
def cache_api():
    return private(CACHE, "f03323_gles_normative_pdf_cache")
def source_input(cache_root: Path, source_class: str = LIMIT_CLASS) -> dict[str, object]:
    if source_class != LIMIT_CLASS:
        reject("only the admitted GLES limit-format source class may enter this inventory")
    cache = cache_api()
    try:
        inspected = cache.inspect(cache_root, LOCATOR, source_class)
        authority, manifest = cache.SOURCE_API.authority(), cache.manifest()
        boundary, decision = authority.validate(), authority.consume(source_class, LOCATOR)["decision"]
        raw = cache.SOURCE_API.pdf_bytes(cache.SOURCE_API.external_root(cache_root), inspected["source"])
        pages = cache.SOURCE_API.physical_pages(raw)
    except Exception as error:
        reject(str(error))
    if not exact(inspected["source"], SOURCE) or pages != PAGES:
        reject("sealed GLES cache did not return the exact normative PDF")
    return {"raw": raw, "source": inspected["source"], "decision": decision,
            "physical_pdf_pages": pages, "cache_boundary_sha256": manifest["cache_boundary_sha256"],
            "cache_layout": manifest["cache_layout"], "source_authority_boundary_sha256": boundary["boundary_sha256"],
            "source_contract_sha256": boundary["source_contract_sha256"], "inventory_lock_sha256": boundary["inventory_lock_sha256"]}
def rows() -> list[dict[str, object]]:
    result = []
    for table, page, title, text in TABLES:
        for number, line in enumerate(text.strip().splitlines(), 1):
            route, kind, section, name = line.split("|", 3)
            result.append({"table": table, "physical_page": page, "title": title, "table_row": number,
                           "route": route, "kind": kind, "numeric_section": section, "name": name})
    return result
def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-", "-"], input=raw,
                                capture_output=True, check=False)
    except OSError as error:
        reject(f"pdftotext is required for sealed PDF anchors: {error}")
    if result.returncode != 0:
        reject("sealed PDF anchor extraction failed")
    try:
        return result.stdout.decode("utf-8")
    except UnicodeDecodeError as error:
        reject(f"sealed PDF anchors are not UTF-8: {error}")
def line_count(text: str, anchor: str) -> int:
    return sum(line.strip() == anchor for line in text.splitlines())
def coverage() -> dict[str, object]:
    reviewed = []
    for table, page, _, _ in TABLES:
        group = [row for row in rows() if row["table"] == table]
        included = [row["table_row"] for row in group if row["route"] == "admit"]
        excluded = [{"row": row["table_row"], "name": row["name"], "numeric_section": row["numeric_section"],
                     "reason": row["kind"], "route": row["route"]} for row in group if row["route"] != "admit"]
        reviewed.append({"table": table, "physical_page": page, "included_rows": included, "excluded_rows": excluded})
    return {"manifest_id": "gles32-reviewed-limit-format-tables-21-40-through-21-42-v1", "complete": False,
            "coverage_decision": "bounded-reviewed-table-rows-only", "reviewed_tables": reviewed,
            "unadmitted_inputs": ["shader", "precision", "extension", "essl-320-spec", "gl.xml", "desktop-glsl", "gles-3.1"],
            "unreviewed_families": ["implementation-dependent tables outside-21.40-through-21.42-unclassified",
                                      "chapter-local-limit-and-format-rules-outside-reviewed-table-scope-unclassified"]}
def facts(raw: bytes, pages: int) -> list[dict[str, object]]:
    if pages != PAGES:
        reject("sealed PDF has an unexpected physical page count")
    output, source_order = [], 1
    for table, page, title, _ in TABLES:
        text, group = page_text(raw, page), [row for row in rows() if row["table"] == table]
        if line_count(text, title) != 1:
            reject("sealed PDF has a missing or ambiguous implementation-value table anchor")
        for row in group:
            if line_count(text, row["name"]) != 1:
                reject("sealed PDF has a missing, ambiguous, or reordered table row anchor")
            if row["route"] == "admit":
                output.append({"raw_id": f"gles32-table-{table.replace('.', '-')}-row-{row['table_row']:02d}",
                               "name": row["name"], "kind": row["kind"], "physical_page": page,
                               "numeric_section": row["numeric_section"],
                               "source_locator": f"gles32-pdf-v1:page={page};section={row['numeric_section']}",
                               "table": table, "table_row": row["table_row"], "source_order": source_order,
                               "derivation_class": LIMIT_CLASS})
                source_order += 1
        if table == "21.42" and line_count(text, "GetShaderPrecisionFormat") != 2:
            reject("sealed PDF has a missing or ambiguous precision-row anchor")
    if len(rows()) != 46 or len(output) != 34 or [row["source_order"] for row in output] != list(range(1, 35)):
        reject("bounded table slice lost ordered source coverage")
    return output
