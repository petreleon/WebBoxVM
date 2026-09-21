"""Bounded formal GLES Uniform matrix templates for F03.3.2.2.3.3.4."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "program-pipeline-commands"
FAMILY = (
    "object-declarations", "program-shader",
    ("7.1-7.11.1", "7.11.3-7.13.1", "7.13.3-7.15"), 15, 85, "7.1",
    "uint CreateShader( enum type );",
)
SOURCE_PAGES = (126,)
SECTION = "7.6.1"
SECTION_WITNESS = "7.6.1 Loading Uniform Variables In The Default Uniform Block"
FORMAL_WINDOW = (
    "void Uniform{1234}{if ui}( int location, T value );",
    "void Uniform{1234}{if ui}v( int location, sizei count, const T *value );",
    "void UniformMatrix{234}fv( int location, sizei count, boolean transpose, const float *value );",
    "void UniformMatrix{2x3,3x2,2x4,4x2,3x4,4x3}fv( int location, sizei count, boolean transpose, const float *value );",
)
TEMPLATES = tuple(enumerate(FORMAL_WINDOW[2:], 4))
WINDOW_META = ((2, 12), (3, 12), (4, 3), (5, 6))
MATRIX_SUFFIXES = ("2", "3", "4", "2x3", "3x2", "2x4", "4x2", "3x4", "4x3")
EXPANDED_NAMES = tuple("UniformMatrix" + suffix + "fv" for suffix in MATRIX_SUFFIXES)
SEALED_WINDOW_SHA256 = "0876e1fbf81c6ecd8961819dc304f7e3ab7ad0d996a85cd1ec9f297df76fe3f9"
SEALED_TEMPLATES_SHA256 = "30fa2d78aa46a8bb3e4f9f908855519efb9f16a48330a62b9aeb43f0809becfd"
SOURCE_TEMPLATE = re.compile(r"void [A-Z][A-Za-z0-9]*(?:\{[^{}]+\}[A-Za-z0-9]*)+\([^;]*\);")
NAME = re.compile(r"void ([A-Z][A-Za-z0-9]*(?:\{[^{}]+\}[A-Za-z0-9]*)+)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A proposed template lies outside the sealed Uniform matrix slice."""


def reject(message: str) -> None:
    raise CatalogError(message)


def compact(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def template_sha256(rows: object) -> str:
    return hashlib.sha256(json.dumps(rows, separators=(",", ":"), ensure_ascii=True).encode("utf-8")).hexdigest()


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw,
                                capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed Uniform matrix template page: {error}")
    if result.returncode != 0:
        reject("cannot read sealed Uniform matrix template page")
    return compact(value)


def source_slice(value: str) -> tuple[str, ...]:
    return tuple(SOURCE_TEMPLATE.findall(value))


def template_name(declaration: str) -> str:
    match = NAME.fullmatch(declaration)
    if match is None:
        reject("formal Uniform matrix template spelling is malformed")
    return match.group(1)


def pretty_json(value: dict[str, object]) -> str:
    """Keep expanded template rows readable without breaking the file-size cap."""
    lines, pairs = ["{"], sorted(value.items())
    for position, (key, item) in enumerate(pairs):
        comma = "," if position + 1 < len(pairs) else ""
        if key != "raw_entries":
            field = f"{json.dumps(key)}: {json.dumps(item, indent=2, sort_keys=True)}"
            chunk = [f"  {line}" for line in field.splitlines()]
            chunk[-1] += comma
            lines.extend(chunk)
            continue
        if not isinstance(item, list) or not all(isinstance(row, list) and len(row) == 7 for row in item):
            reject("Uniform matrix inventory rows cannot be rendered")
        lines.append('  "raw_entries": [')
        for row_number, row in enumerate(item):
            row_comma = "," if row_number + 1 < len(item) else ""
            lines.extend((f"    [{json.dumps(row[0])}, {json.dumps(row[1])}, {json.dumps(row[2])},",
                          f"     {json.dumps(row[3])},",
                          f"     {json.dumps(row[4])}, {json.dumps(row[5])}, {json.dumps(row[6])}]{row_comma}"))
        lines.append(f"  ]{comma}")
    return "\n".join((*lines, "}"))


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    value = document(chunk_paths[chunk])
    rows = [row for row in value.get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected:
        reject("Uniform matrix command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def grammar_window(grammar: dict[str, object]) -> None:
    try:
        forms = grammar["grammar"]["formal_templates"]
        expected = [{"formal_name": template_name(declaration), "physical_page": 126, "section": SECTION,
                     "source_order": order, "expansion_count": count}
                    for (order, count), declaration in zip(WINDOW_META, FORMAL_WINDOW)]
    except (KeyError, TypeError, AttributeError):
        reject("sealed declaration grammar template catalog is malformed")
    actual = [item for item in forms if item.get("physical_page") == 126 and item.get("section") == SECTION]
    if actual != expected:
        reject("Uniform matrix grammar window is stale, incomplete, rerouted, or promoted")


def facts(raw: bytes, pages: int, family_order: int, grammar: dict[str, object], normalize) -> list[list[object]]:
    if (pages != PAGES or family_order != FAMILY[3] or SOURCE_PAGES != (126,)
            or template_sha256(FORMAL_WINDOW) != SEALED_WINDOW_SHA256
            or template_sha256(TEMPLATES) != SEALED_TEMPLATES_SHA256
            or TEMPLATES != tuple(enumerate(FORMAL_WINDOW[2:], 4))):
        reject("GLES Uniform matrix source boundary, literal set, locations, or order is incomplete")
    text = page_text(raw, 126)
    if SECTION_WITNESS not in text or source_slice(text) != FORMAL_WINDOW:
        reject("formal Uniform matrix template source window is incomplete, rerouted, or out of source order")
    grammar_window(grammar)
    result, names, offset = [], set(), 0
    for template_order, declaration in TEMPLATES:
        formal, start = template_name(declaration), text.find(declaration)
        if start <= text.find(SECTION_WITNESS) or declaration not in text or "Matrix" not in formal or "Program" in formal:
            reject("formal Uniform matrix template is missing, malformed, or outside this source slice")
        try:
            c_names = normalize([("template", formal)])
        except Exception as error:
            reject(f"declaration grammar rejected a Uniform matrix template: {error}")
        count = dict(WINDOW_META)[template_order]
        expected = ["gl" + name for name in EXPANDED_NAMES[offset:offset + count]]
        if c_names != expected:
            reject("Uniform matrix template expansion is incomplete, guessed, or cross-family")
        offset += count
        for c_name in c_names:
            name, order = c_name[2:], len(result) + 1
            if name in names:
                reject("expanded Uniform matrix spelling is duplicated")
            names.add(name)
            result.append([f"gles32-uniform-matrix-{order:02d}", name, c_name, declaration, 126, SECTION, order])
    if len(result) != 9 or [row[-1] for row in result] != list(range(1, 10)):
        reject("expanded Uniform matrix ordering is incomplete or unstable")
    return result
