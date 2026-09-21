"""Bounded formal GLES texture/sampler object declarations for F03.3.2.2.3.4.1."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess

PROFILE, PAGES, ROUTE = "gles-3.2", 601, "texture-sampler-commands"
FAMILIES = (
    ("object-declarations", "texture", ("8.1", "8.5-8.26"), 17, 157, "8.1", "void GenTextures( sizei n, uint *textures );;"),
    ("object-declarations", "sampler", ("8.2",), 18, 159, "8.2", "void GenSamplers( sizei count, uint *samplers );"),
)
SOURCE_PAGES = (157, 158, 159, 160, 161)
SECTION_WITNESSES = {"8.1": (156, "8.1 Texture Objects"), "8.2": (158, "8.2 Sampler Objects")}
SECTION_PAGES = {"8.1": (157, 158), "8.2": (159, 160, 161)}
SECTION_BOUNDARIES = {(158, "8.1"): ("before", "8.2 Sampler Objects"),
                      (161, "8.2"): ("before", "8.3 Sampler Object Queries")}
FORMS = (
    ("texture", 157, "8.1", "literal", "GenTextures", "void GenTextures( sizei n, uint *textures );;"),
    ("texture", 157, "8.1", "literal", "BindTexture", "void BindTexture( enum target, uint texture );"),
    ("texture", 158, "8.1", "literal", "DeleteTextures", "void DeleteTextures( sizei n, const uint *textures );"),
    ("texture", 158, "8.1", "literal", "IsTexture", "boolean IsTexture( uint texture );"),
    ("sampler", 159, "8.2", "literal", "GenSamplers", "void GenSamplers( sizei count, uint *samplers );"),
    ("sampler", 159, "8.2", "literal", "BindSampler", "void BindSampler( uint unit, uint sampler );"),
    ("sampler", 160, "8.2", "template", "SamplerParameter{if}", "void SamplerParameter{if}( uint sampler, enum pname, T param );"),
    ("sampler", 160, "8.2", "template", "SamplerParameter{if}v", "void SamplerParameter{if}v( uint sampler, enum pname, const T *params );"),
    ("sampler", 160, "8.2", "template", "SamplerParameterI{i ui}v", "void SamplerParameterI{i ui}v( uint sampler, enum pname, const T *params );"),
    ("sampler", 161, "8.2", "literal", "DeleteSamplers", "void DeleteSamplers( sizei count, const uint *samplers );"),
    ("sampler", 161, "8.2", "literal", "IsSampler", "boolean IsSampler( uint sampler );"),
)
TEMPLATE_META = ((12, 2), (13, 2), (14, 2))
TEMPLATE_NAMES = tuple(row[4] for row in FORMS if row[3] == "template")
EXPANDED_NAMES = ("SamplerParameteri", "SamplerParameterf", "SamplerParameteriv", "SamplerParameterfv",
                  "SamplerParameterIiv", "SamplerParameterIuiv")
SEALED_FORMS_SHA256 = "a782182085a0a3564cc4a2248e02ec31ffde8f8202741845a534b3df843cd312"
SEALED_TEMPLATES_SHA256 = "9a23ec9ebe71c64bce305ad25f8b78a007d61a879b79406db3990906fd537991"
SOURCE_FORMAL = re.compile(r"(?:void|boolean) [A-Z][A-Za-z0-9]*(?:\{[^{}]+\}[A-Za-z0-9]*)*\([^;]*\);;?")
LITERAL = re.compile(r"(?:void|boolean) ([A-Z][A-Za-z0-9]*)\([^;]*\);;?\Z")
TEMPLATE = re.compile(r"void ([A-Z][A-Za-z0-9]*(?:\{[^{}]+\}[A-Za-z0-9]*)+)\([^;]*\);\Z")


class CatalogError(ValueError):
    """A proposed declaration lies outside the sealed texture/sampler object slice."""


def reject(message: str) -> None:
    raise CatalogError(message)


def compact(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def sha256(value: object) -> str:
    return hashlib.sha256(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode("utf-8")).hexdigest()


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw,
                                capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed texture/sampler object page: {error}")
    if result.returncode != 0:
        reject("cannot read sealed texture/sampler object page")
    return compact(value)


def source_slice(text) -> tuple[tuple[int, str], ...]:
    return tuple((page, item) for page in SOURCE_PAGES for item in SOURCE_FORMAL.findall(text(page)))


def in_section_window(page: int, section: str, declaration: str, value: str) -> bool:
    boundary = SECTION_BOUNDARIES.get((page, section))
    if boundary is None:
        return True
    relation, marker = boundary
    start, split = value.find(declaration), value.find(marker)
    return start >= 0 and split >= 0 and ((relation == "before" and start < split) or (relation == "after" and start > split))


def bound_families(chunk_paths: dict[str, object], document) -> dict[str, int]:
    result = {}
    for chunk, identifier, scopes, order, page, section, anchor in FAMILIES:
        rows = [row for row in document(chunk_paths[chunk]).get("families", []) if row.get("id") == identifier]
        expected = {"id": identifier, "family_kind": "declaration", "source_scope": list(scopes), "route": ROUTE,
                    "reason": "formal-declaration-only", "source_order": order,
                    "anchor": {"kind": "declaration", "physical_page": page, "section": section, "text": anchor}}
        if len(rows) != 1 or rows[0] != expected:
            reject("texture/sampler command-domain binding is stale, incomplete, rerouted, or promoted")
        result[identifier] = order
    if result != {"texture": 17, "sampler": 18}:
        reject("texture/sampler command-domain families are incomplete or cross-routed")
    return result


def grammar_window(grammar: dict[str, object]) -> None:
    try:
        forms = grammar["grammar"]["formal_templates"]
        expected = [{"formal_name": name, "physical_page": 160, "section": "8.2", "source_order": order,
                     "expansion_count": count} for name, (order, count) in zip(TEMPLATE_NAMES, TEMPLATE_META)]
    except (KeyError, TypeError, AttributeError):
        reject("sealed declaration grammar template catalog is malformed")
    actual = [row for row in forms if row.get("physical_page") == 160 and row.get("section") == "8.2"]
    if actual != expected:
        reject("sampler-parameter grammar window is stale, incomplete, rerouted, or promoted")


def facts(raw: bytes, pages: int, family_orders: dict[str, int], grammar: dict[str, object], normalize) -> list[list[object]]:
    fixed_boundaries = {(158, "8.1"): ("before", "8.2 Sampler Objects"), (161, "8.2"): ("before", "8.3 Sampler Object Queries")}
    if (pages != PAGES or family_orders != {"texture": 17, "sampler": 18} or SOURCE_PAGES != (157, 158, 159, 160, 161)
            or SECTION_WITNESSES != {"8.1": (156, "8.1 Texture Objects"), "8.2": (158, "8.2 Sampler Objects")}
            or SECTION_PAGES != {"8.1": (157, 158), "8.2": (159, 160, 161)} or SECTION_BOUNDARIES != fixed_boundaries
            or sha256(FORMS) != SEALED_FORMS_SHA256 or sha256(TEMPLATE_META) != SEALED_TEMPLATES_SHA256):
        reject("GLES texture/sampler object source boundary, locations, or order is incomplete")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page))
        return texts[page]
    expected_slice = tuple((row[1], row[5]) for row in FORMS)
    if source_slice(text) != expected_slice:
        reject("formal texture/sampler object source window is incomplete, rerouted, or out of source order")
    grammar_window(grammar)
    result, names, offset = [], set(), 0
    for family, page, section, kind, name, declaration in FORMS:
        witness, value = SECTION_WITNESSES.get(section), text(page)
        if (witness is None or page not in SECTION_PAGES.get(section, ()) or witness[1] not in text(witness[0])
                or declaration not in value or not in_section_window(page, section, declaration, value)):
            reject("texture/sampler declaration or section boundary is missing or outside this source slice")
        match = LITERAL.fullmatch(declaration) if kind == "literal" else TEMPLATE.fullmatch(declaration)
        if match is None or match.group(1) != name:
            reject("texture/sampler declaration spelling is malformed")
        try:
            c_names = normalize([(kind, name)])
        except Exception as error:
            reject(f"declaration grammar rejected a texture/sampler form: {error}")
        if kind == "literal":
            expected, derivation = ["gl" + name], "literal"
        else:
            count = TEMPLATE_META[offset][1]; expected = ["gl" + item for item in EXPANDED_NAMES[sum(item[1] for item in TEMPLATE_META[:offset]):sum(item[1] for item in TEMPLATE_META[:offset + 1])]]
            derivation = "template-expansion"; offset += 1
            if len(expected) != count:
                reject("sampler-parameter expansion count is stale")
        if c_names != expected or any(item[2:] in names for item in c_names):
            reject("texture/sampler declaration expansion is duplicate, guessed, or cross-family")
        for c_name in c_names:
            names.add(c_name[2:]); order = len(result) + 1
            result.append([f"gles32-texture-sampler-object-parameter-{order:02d}", family, family_orders[family], c_name[2:],
                           c_name, declaration, page, section, order, derivation])
    if len(result) != 14 or [row[8] for row in result] != list(range(1, 15)):
        reject("texture/sampler object raw ordering is incomplete or unstable")
    return result
