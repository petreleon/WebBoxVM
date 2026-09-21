"""Bounded GLES current-generic-attribute templates for F03.3.2.2.3.6.1."""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
from pathlib import Path

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "vertex-transform-feedback-commands"
FAMILY = ("object-declarations", "vertex-array", ("10.2", "10.3.1-10.3.8", "10.4"), 22,
          294, "10.4", "void GenVertexArrays( sizei n, uint *arrays );")
SOURCE_PAGE, SECTION = 283, "10.2.1"
SECTION_WITNESS = "10.2.1 Current Generic Attributes"
FORMAL_WINDOW = (
    "void VertexAttrib{1234}f( uint index,float values );",
    "void VertexAttrib{1234}fv( uint index,const float *values );",
    "void VertexAttribI4{i ui}( uint index, T values );",
    "void VertexAttribI4{i ui}v( uint index, const T values );",
)
EXPANDED_NAMES = (
    ("VertexAttrib1f", "VertexAttrib2f", "VertexAttrib3f", "VertexAttrib4f"),
    ("VertexAttrib1fv", "VertexAttrib2fv", "VertexAttrib3fv", "VertexAttrib4fv"),
    ("VertexAttribI4i", "VertexAttribI4ui"), ("VertexAttribI4iv", "VertexAttribI4uiv"),
)
TEMPLATE_ORDERS = (23, 24, 25, 26)
TEMPLATES = tuple(zip(TEMPLATE_ORDERS, FORMAL_WINDOW, EXPANDED_NAMES))
SEALED_WINDOW_SHA256 = "2520a6eefaa9ffae4e314c352275da1e85fe3dea0c75947810219407b3c3aeac"
TRUSTED_EXTRACTORS = (Path("/opt/homebrew/bin/pdftotext"), Path("/usr/local/bin/pdftotext"), Path("/usr/bin/pdftotext"))
SOURCE_TEMPLATE = re.compile(r"void [A-Z][A-Za-z0-9]*(?:\{[^{}]+\}[A-Za-z0-9]*)+\([^;]*\);")
NAME = re.compile(r"void ([A-Z][A-Za-z0-9]*(?:\{[^{}]+\}[A-Za-z0-9]*)+)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A template lies outside the sealed current-vertex-attribute slice."""


def reject(message: str) -> None: raise CatalogError(message)


def compact(value: str) -> str: return " ".join(value.replace("\f", " ").split())


def template_sha256(rows: object) -> str:
    return hashlib.sha256(json.dumps(rows, separators=(",", ":"), ensure_ascii=True).encode("utf-8")).hexdigest()


def extractor() -> str:
    for candidate in TRUSTED_EXTRACTORS:
        resolved = candidate.resolve()
        if resolved.is_file() and os.access(resolved, os.X_OK): return str(resolved)
    reject("fixed PDF text extractor is unavailable")


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run([extractor(), "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw, capture_output=True, check=False,
                                env={"LC_ALL": "C", "PATH": "/usr/bin:/bin"})
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error: reject(f"cannot read sealed current-vertex-attribute page: {error}")
    if result.returncode != 0: reject("cannot read sealed current-vertex-attribute page")
    return compact(value)


def source_slice(value: str) -> tuple[str, ...]: return tuple(SOURCE_TEMPLATE.findall(value))


def template_name(declaration: str) -> str:
    match = NAME.fullmatch(declaration)
    if match is None: reject("formal current-vertex-attribute template spelling is malformed")
    return match.group(1)


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    rows = [row for row in document(chunk_paths[chunk]).get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected: reject("current-vertex-attribute domain binding is stale, incomplete, rerouted, or promoted")
    return order


def grammar_window(grammar: dict[str, object]) -> None:
    try:
        expected = [{"formal_name": template_name(declaration), "physical_page": SOURCE_PAGE, "section": SECTION,
                     "source_order": order, "expansion_count": len(names)} for order, declaration, names in TEMPLATES]
        actual = [item for item in grammar["grammar"]["formal_templates"] if item.get("physical_page") == SOURCE_PAGE and item.get("section") == SECTION]
    except (KeyError, TypeError): reject("sealed declaration grammar template catalog is malformed")
    if actual != expected: reject("current-vertex-attribute grammar window is stale, incomplete, rerouted, or promoted")


def facts(raw: bytes, pages: int, family_order: int, grammar: dict[str, object], normalize) -> list[list[object]]:
    fixed_window = (
        "void VertexAttrib{1234}f( uint index,float values );",
        "void VertexAttrib{1234}fv( uint index,const float *values );",
        "void VertexAttribI4{i ui}( uint index, T values );",
        "void VertexAttribI4{i ui}v( uint index, const T values );",
    )
    fixed_names = (("VertexAttrib1f", "VertexAttrib2f", "VertexAttrib3f", "VertexAttrib4f"),
                   ("VertexAttrib1fv", "VertexAttrib2fv", "VertexAttrib3fv", "VertexAttrib4fv"),
                   ("VertexAttribI4i", "VertexAttribI4ui"), ("VertexAttribI4iv", "VertexAttribI4uiv"))
    if (PROFILE != "gles-3.2" or pages != PAGES or family_order != 22 or SOURCE_PAGE != 283 or SECTION != "10.2.1"
            or SECTION_WITNESS != "10.2.1 Current Generic Attributes" or FORMAL_WINDOW != fixed_window
            or EXPANDED_NAMES != fixed_names or TEMPLATE_ORDERS != (23, 24, 25, 26)
            or TEMPLATES != tuple(zip(TEMPLATE_ORDERS, FORMAL_WINDOW, EXPANDED_NAMES))
            or template_sha256(FORMAL_WINDOW) != SEALED_WINDOW_SHA256):
        reject("GLES current-vertex-attribute source boundary, locations, or order is incomplete")
    text = page_text(raw, SOURCE_PAGE)
    if SECTION_WITNESS not in text or source_slice(text) != FORMAL_WINDOW:
        reject("current-vertex-attribute template source window is incomplete, rerouted, or out of source order")
    grammar_window(grammar)
    result, names = [], set()
    for template_order, declaration, expected_names in TEMPLATES:
        formal, start = template_name(declaration), text.find(declaration)
        if start <= text.find(SECTION_WITNESS) or declaration not in text: reject("current-vertex-attribute template is missing or outside this source slice")
        try: c_names = normalize([("template", formal)])
        except Exception as error: reject(f"declaration grammar rejected a current-vertex-attribute template: {error}")
        if c_names != ["gl" + name for name in expected_names]: reject("current-vertex-attribute template expansion is incomplete, guessed, or cross-family")
        for c_name in c_names:
            name, order = c_name[2:], len(result) + 1
            if name in names: reject("expanded current-vertex-attribute spelling is duplicated")
            names.add(name); result.append([f"gles32-current-vertex-attribute-{order:02d}", name, c_name, declaration, SOURCE_PAGE, SECTION, order])
    if len(result) != 12 or [row[-1] for row in result] != list(range(1, 13)):
        reject("expanded current-vertex-attribute ordering is incomplete or unstable")
    return result
