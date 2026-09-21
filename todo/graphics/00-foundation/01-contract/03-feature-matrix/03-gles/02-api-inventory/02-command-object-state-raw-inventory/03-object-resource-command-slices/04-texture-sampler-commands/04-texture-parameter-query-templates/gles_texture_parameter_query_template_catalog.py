"""Bounded GLES texture parameter/query templates for F03.3.2.2.3.4.4."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "texture-sampler-commands"
FAMILY = ("object-declarations", "texture", ("8.1", "8.5-8.26"), 17, 157, "8.1", "void GenTextures( sizei n, uint *textures );;")
SOURCE_PAGES = (206, 209, 210)
SECTION_WITNESSES = {
    "8.10": (206, "8.10 Texture Parameters"), "8.11.2": (209, "8.11.2 Texture Parameter Queries"),
    "8.11.3": (210, "8.11.3 Texture Level Parameter Queries"),
}
FORMAL_WINDOW = (
    (206, "8.10", "void TexParameter{if}( enum target, enum pname, T param );"),
    (206, "8.10", "void TexParameter{if}v( enum target, enum pname, const T *params );"),
    (206, "8.10", "void TexParameterI{i ui}v( uint texture, enum pname, const T *params );"),
    (209, "8.11.2", "void GetTexParameter{if}v( enum target, enum pname, T *params );"),
    (209, "8.11.2", "void GetTexParameterI{i ui}v( enum target, enum pname, T *params );"),
    (210, "8.11.3", "void GetTexLevelParameter{if}v( enum target, int level, enum pname, T *params );"),
)
WINDOW_META = ((17, 2), (18, 2), (19, 2), (20, 2), (21, 2), (22, 2))
EXPANDED_NAMES = (
    ("TexParameteri", "TexParameterf"), ("TexParameteriv", "TexParameterfv"),
    ("TexParameterIiv", "TexParameterIuiv"), ("GetTexParameteriv", "GetTexParameterfv"),
    ("GetTexParameterIiv", "GetTexParameterIuiv"),
    ("GetTexLevelParameteriv", "GetTexLevelParameterfv"),
)
SEALED_WINDOW_SHA256 = "6bacf7cda6112ad2ca3afa03dfed2f7ae089014af2a6a95173e67e8708c5343b"
SEALED_EXPANSIONS_SHA256 = "6ea64ff5eeea7313dcdaa0aa1902ffe2f9532ae1270d50a366615b289d12e91e"
SOURCE_TEMPLATE = re.compile(r"void [A-Z][A-Za-z0-9]*(?:\{[^{}]+\}[A-Za-z0-9]*)+\([^;]*\);")
NAME = re.compile(r"void ([A-Z][A-Za-z0-9]*(?:\{[^{}]+\}[A-Za-z0-9]*)+)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A template lies outside the sealed texture parameter/query slice."""


def reject(message: str) -> None: raise CatalogError(message)


def compact(value: str) -> str: return " ".join(value.replace("\f", " ").split())


def sha256(value: object) -> str:
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode("utf-8")).hexdigest()


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw, capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error: reject(f"cannot read sealed texture parameter/query page: {error}")
    if result.returncode != 0: reject("cannot read sealed texture parameter/query page")
    return compact(value)


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in SOURCE_PAGES for item in SOURCE_TEMPLATE.findall(text(page)))


def template_name(declaration: str) -> str:
    match = NAME.fullmatch(declaration)
    if match is None: reject("formal texture parameter/query template spelling is malformed")
    return match.group(1)


def bound_family(chunk_paths: dict[str, object], document) -> int:
    chunk, identifier, scopes, order, page, section, anchor = FAMILY
    rows = [row for row in document(chunk_paths[chunk]).get("families", []) if row.get("id") == identifier]
    expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                "reason": "formal-declaration-only", "source_order": order,
                "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
    if len(rows) != 1 or rows[0] != expected: reject("texture command-domain binding is stale, incomplete, rerouted, or promoted")
    return order


def grammar_window(grammar: dict[str, object]) -> None:
    try:
        forms = grammar["grammar"]["formal_templates"]
        expected = [{"formal_name": template_name(declaration), "physical_page": page, "section": section,
                     "source_order": order, "expansion_count": count}
                    for ((page, section, declaration), (order, count)) in zip(FORMAL_WINDOW, WINDOW_META)]
    except (KeyError, TypeError): reject("sealed declaration grammar template catalog is malformed")
    locations = {(page, section) for page, section, _ in FORMAL_WINDOW}
    actual = [item for item in forms if (item.get("physical_page"), item.get("section")) in locations]
    if actual != expected: reject("texture parameter/query grammar window is stale, incomplete, rerouted, or promoted")


def facts(raw: bytes, pages: int, family_order: int, grammar: dict[str, object], normalize) -> list[list[object]]:
    fixed_witnesses = {"8.10": (206, "8.10 Texture Parameters"), "8.11.2": (209, "8.11.2 Texture Parameter Queries"), "8.11.3": (210, "8.11.3 Texture Level Parameter Queries")}
    if (pages != PAGES or family_order != 17 or SOURCE_PAGES != (206, 209, 210) or SECTION_WITNESSES != fixed_witnesses
            or sha256(FORMAL_WINDOW) != SEALED_WINDOW_SHA256 or sha256(EXPANDED_NAMES) != SEALED_EXPANSIONS_SHA256):
        reject("GLES texture parameter/query source boundary, forms, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page)); return texts[page]
    expected = tuple((page, declaration) for page, _, declaration in FORMAL_WINDOW)
    if source_slice(text) != expected: reject("texture parameter/query source window is incomplete, rerouted, or out of source order")
    grammar_window(grammar); result, names = [], set()
    for (page, section, declaration), expected_names in zip(FORMAL_WINDOW, EXPANDED_NAMES):
        formal, witness, value = template_name(declaration), SECTION_WITNESSES[section], text(page)
        if (declaration not in value or witness[1] not in text(witness[0]) or value.find(declaration) <= value.find(witness[1])
                or "Sampler" in formal or not formal.startswith(("Tex", "GetTex"))):
            reject("formal texture parameter/query template is missing, malformed, or outside this source slice")
        try: c_names = normalize([("template", formal)])
        except Exception as error: reject(f"declaration grammar rejected a texture parameter/query template: {error}")
        if c_names != ["gl" + name for name in expected_names]: reject("texture parameter/query expansion is incomplete, guessed, or cross-family")
        for c_name in c_names:
            name, order = c_name[2:], len(result) + 1
            if name in names: reject("expanded texture parameter/query spelling is duplicated")
            names.add(name); result.append([f"gles32-texture-parameter-query-{order:02d}", name, c_name, declaration, page, section, order])
    if len(result) != 12 or [row[-1] for row in result] != list(range(1, 13)): reject("expanded texture parameter/query ordering is incomplete or unstable")
    return result
