"""Bounded formal GLES ProgramUniform scalar/vector templates for F03.3.2.2.3.3.5."""

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
SOURCE_PAGES = (128,)
SECTION = "7.6.1"
SECTION_WITNESS = (126, "7.6.1 Loading Uniform Variables In The Default Uniform Block")
FORMAL_WINDOW = (
    "void ProgramUniform{1234}{if}( uint program, int location, T value );",
    "void ProgramUniform{1234}{if}v( uint program, int location, sizei count, const T *value );",
    "void ProgramUniform{1234}ui( uint program, int location, T value );",
    "void ProgramUniform{1234}uiv( uint program, int location, sizei count, const T *value );",
)
TEMPLATES = tuple(enumerate(FORMAL_WINDOW, 6))
WINDOW_META = ((6, 8), (7, 8), (8, 4), (9, 4))
SCALAR_SUFFIXES = tuple(str(size) + kind for size in "1234" for kind in ("i", "f"))
UNSIGNED_SUFFIXES = tuple(str(size) + "ui" for size in "1234")
EXPANDED_NAMES = (tuple("ProgramUniform" + suffix for suffix in SCALAR_SUFFIXES)
                  + tuple("ProgramUniform" + suffix + "v" for suffix in SCALAR_SUFFIXES)
                  + tuple("ProgramUniform" + suffix for suffix in UNSIGNED_SUFFIXES)
                  + tuple("ProgramUniform" + suffix + "v" for suffix in UNSIGNED_SUFFIXES))
SEALED_WINDOW_SHA256 = "3b98c36e4cb053b684e620f7b221a3bd5c9c4040358d6a48b6e5e63f8b997874"
SEALED_TEMPLATES_SHA256 = "d30417643609417bab69d022e1b68723ac1eda2c2274c3ca1b40e4265c2fcbf2"
SOURCE_TEMPLATE = re.compile(r"void [A-Z][A-Za-z0-9]*(?:\{[^{}]+\}[A-Za-z0-9]*)+\([^;]*\);")
NAME = re.compile(r"void ([A-Z][A-Za-z0-9]*(?:\{[^{}]+\}[A-Za-z0-9]*)+)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A proposed template lies outside the sealed ProgramUniform scalar/vector slice."""


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
        reject(f"cannot read sealed ProgramUniform template page: {error}")
    if result.returncode != 0:
        reject("cannot read sealed ProgramUniform template page")
    return compact(value)


def source_slice(value: str) -> tuple[str, ...]:
    return tuple(SOURCE_TEMPLATE.findall(value))


def template_name(declaration: str) -> str:
    match = NAME.fullmatch(declaration)
    if match is None:
        reject("formal ProgramUniform template spelling is malformed")
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
            reject("ProgramUniform scalar/vector inventory rows cannot be rendered")
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
        reject("ProgramUniform scalar/vector command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def grammar_window(grammar: dict[str, object]) -> None:
    try:
        forms = grammar["grammar"]["formal_templates"]
        expected = [{"formal_name": template_name(declaration), "physical_page": 128, "section": SECTION,
                     "source_order": order, "expansion_count": count}
                    for (order, count), declaration in zip(WINDOW_META, FORMAL_WINDOW)]
    except (KeyError, TypeError, AttributeError):
        reject("sealed declaration grammar template catalog is malformed")
    actual = [item for item in forms if item.get("physical_page") == 128 and item.get("section") == SECTION]
    if actual != expected:
        reject("ProgramUniform scalar/vector grammar window is stale, incomplete, rerouted, or promoted")


def facts(raw: bytes, pages: int, family_order: int, grammar: dict[str, object], normalize) -> list[list[object]]:
    if (pages != PAGES or family_order != FAMILY[3] or SOURCE_PAGES != (128,)
            or template_sha256(FORMAL_WINDOW) != SEALED_WINDOW_SHA256
            or template_sha256(TEMPLATES) != SEALED_TEMPLATES_SHA256
            or TEMPLATES != tuple(enumerate(FORMAL_WINDOW, 6))):
        reject("GLES ProgramUniform scalar/vector source boundary, literal set, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page))
        return texts[page]
    if SECTION_WITNESS[1] not in text(SECTION_WITNESS[0]) or source_slice(text(128)) != FORMAL_WINDOW:
        reject("formal ProgramUniform template source window is incomplete, rerouted, or out of source order")
    grammar_window(grammar)
    result, names, offset = [], set(), 0
    for template_order, declaration in TEMPLATES:
        formal = template_name(declaration)
        if declaration not in text(128) or not formal.startswith("ProgramUniform") or "Matrix" in formal:
            reject("formal ProgramUniform scalar/vector template is missing, malformed, or outside this source slice")
        try:
            c_names = normalize([("template", formal)])
        except Exception as error:
            reject(f"declaration grammar rejected a ProgramUniform template: {error}")
        count = dict(WINDOW_META)[template_order]
        expected = ["gl" + name for name in EXPANDED_NAMES[offset:offset + count]]
        if c_names != expected:
            reject("ProgramUniform template expansion is incomplete, guessed, or cross-family")
        offset += count
        for c_name in c_names:
            name, order = c_name[2:], len(result) + 1
            if name in names:
                reject("expanded ProgramUniform scalar/vector spelling is duplicated")
            names.add(name)
            result.append([f"gles32-program-uniform-scalar-vector-{order:02d}", name, c_name, declaration,
                           128, SECTION, order])
    if len(result) != 24 or [row[-1] for row in result] != list(range(1, 25)):
        reject("expanded ProgramUniform scalar/vector ordering is incomplete or unstable")
    return result
