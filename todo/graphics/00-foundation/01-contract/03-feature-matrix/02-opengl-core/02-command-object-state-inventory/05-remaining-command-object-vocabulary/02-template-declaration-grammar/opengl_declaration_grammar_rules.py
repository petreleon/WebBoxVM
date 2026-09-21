#!/usr/bin/env python3
"""Anchored, fail-closed command-declaration notation for F03.2.2.5.2."""

from __future__ import annotations

import itertools
import re
import subprocess

PROFILE, PAGES, PREFIX = "opengl-4.6-core", 851, "gl"
DESCRIPTORS = (("b", "byte"), ("s", "short"), ("i", "int"), ("i64", "int64"),
               ("f", "float"), ("d", "double"), ("ub", "ubyte"), ("us", "ushort"),
               ("ui", "uint"), ("ui64", "uint64"))
DIGITS, DIMENSIONS, MAX_EXPANSIONS = "1234", ("2x3", "3x2", "2x4", "4x2", "3x4", "4x3"), 64
DECLARATION = re.compile(r"^(?P<rtype>[A-Za-z_][A-Za-z0-9_]*) (?P<name>[A-Za-z_][A-Za-z0-9_{} ,x]*)"
                         r"\( (?P<args>[^{};()]*) \);$")
ANCHORS = (
    (32, "2.1", "c-binding-prefix", "command names, constants, and types are prefixed in the C language binding to OpenGL (by gl, GL_, and GL, respectively), to reduce name clashes with other packages. The prefixes are omitted in this document for clarity."),
    (33, "2.2", "formal-template-notation", "In general, a command declaration has the form"),
    (33, "2.2", "uniform-template", "void Uniform{1234}{if}( int location, T value );"),
    (33, "2.2", "uniform-expansion", "indicates the eight declarations"),
    (34, "2.2", "descriptor-table", "Table 2.1: Correspondence of command suffix type descriptors to GL argument types."),
    (34, "2.2", "continued-uniform-example", "void Uniform4f( int location, float v0, float v1, float v2, float v3 );"),
    (163, "7.6.1", "matrix-square-template", "void UniformMatrix{234}{fd}v( int location, sizei count, boolean transpose, const float *value );"),
    (163, "7.6.1", "matrix-nonsquare-template", "void UniformMatrix{2x3,3x2,2x4,4x2,3x4,4x3}{fd}v( int location, sizei count, boolean transpose, const float *value );"),
)
SECTION_WITNESSES = {"2.1": (30, "2.1 Execution Model"), "2.2": (32, "2.2 Command Syntax"),
                     "7.6.1": (162, "7.6.1 Loading Uniform Variables In The Default Uniform Block")}
LITERALS = ("void Uniform4f( int location, float v0, float v1, float v2, float v3 );",
            "void GetFloatv( enum pname, float *data );")
UNIFORM = "void Uniform{1234}{if}( int location, T value );"


class RuleError(ValueError):
    """A declaration is not a source-anchored, unambiguous formal form."""


def reject(message: str) -> None:
    raise RuleError(message)


def normalized(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-layout", "-", "-"],
                                input=raw, capture_output=True, check=False)
    except OSError as error:
        reject(f"pdftotext is required for formal declaration anchors: {error}")
    if result.returncode:
        reject("formal declaration page extraction failed")
    try:
        return normalized(result.stdout.decode("utf-8"))
    except UnicodeDecodeError as error:
        reject(f"formal declaration page is not UTF-8: {error}")


def require_section(raw: bytes, section: str) -> None:
    witness = SECTION_WITNESSES.get(section)
    if witness is None or witness[1] not in page_text(raw, witness[0]):
        reject("declaration locator does not name its source-section heading")


def formal_pages(raw: bytes) -> dict[int, str]:
    pages = {page: page_text(raw, page) for page in (30, 32, 33, 34, 162, 163)}
    for page, section, label, needle in ANCHORS:
        require_section(raw, section)
        if needle not in pages[page]:
            reject(f"formal declaration anchor is absent: {label}")
        if label == "c-binding-prefix" and pages[page].index(needle) > pages[page].index("2.2 Command Syntax"):
            reject("C binding prefix is not before the §2.2 heading")
    return pages


def parsed(value: str, templated: bool) -> tuple[str, str, str, str]:
    text, match = normalized(value), DECLARATION.fullmatch(normalized(value))
    if match is None or ("{" in text) != templated or ("}" in text) != templated:
        reject("declaration is not the required literal or formal template shape")
    rtype, name, args = match.group("rtype"), match.group("name"), match.group("args")
    if name.startswith(PREFIX) or not args or (not templated and not re.fullmatch(r"[A-Z][A-Za-z0-9]*", name)):
        reject("declaration is prefixed, ambiguous, or not a bare document command name")
    return text, rtype, name, args


def occurrences(text: str, declaration: str) -> int:
    return len(re.findall(rf"(?<![A-Za-z0-9_]){re.escape(declaration)}(?![A-Za-z0-9_])", text))


def anchored(raw: bytes, page: int, section: str, declaration: str, templated: bool):
    text, rtype, name, args = parsed(declaration, templated)
    source = page_text(raw, page)
    require_section(raw, section)
    if occurrences(source, text) != 1:
        reject("formal declaration is absent or ambiguous on its declared PDF page")
    return text, rtype, name, args


def descriptor_choices(value: str) -> tuple[str, ...]:
    compact = "".join(value.split())
    choices = {"".join(token for enabled, (token, _) in zip(mask, DESCRIPTORS) if enabled):
               tuple(token for enabled, (token, _) in zip(mask, DESCRIPTORS) if enabled)
               for mask in itertools.product((False, True), repeat=len(DESCRIPTORS)) if any(mask)}
    if compact not in choices:
        reject("template has a guessed, repeated, reordered, or unknown type suffix")
    return choices[compact]


def group_choices(value: str) -> tuple[str, ...]:
    item = normalized(value)
    if re.fullmatch(r"[1234]+", item):
        if "".join(sorted(set(item))) != item:
            reject("template has a guessed, repeated, or reordered numeric suffix")
        return tuple(item)
    if "," in item:
        result = tuple(part.strip() for part in item.split(","))
        if not result or any(part not in DIMENSIONS for part in result) or tuple(sorted(result, key=DIMENSIONS.index)) != result or len(set(result)) != len(result):
            reject("template has an unknown, repeated, or reordered dimension suffix")
        return result
    return descriptor_choices(item)


def expand_template_name(name: str) -> tuple[str, ...]:
    if name.startswith(PREFIX) or "{" not in name or name.count("{") != name.count("}"):
        reject("template name is prefixed, unexpanded, or malformed")
    pieces, cursor = [], 0
    for match in re.finditer(r"\{([^{}]+)\}", name):
        literal = name[cursor:match.start()]
        if not re.fullmatch(r"[A-Za-z0-9]*", literal):
            reject("template contains an ambiguous suffix")
        pieces.append((literal, group_choices(match.group(1))))
        cursor = match.end()
    tail = name[cursor:]
    if not pieces or not re.fullmatch(r"[A-Za-z0-9]*", tail) or not re.fullmatch(r"[A-Z][A-Za-z0-9]*", pieces[0][0]):
        reject("template has no unambiguous bare command stem")
    options = [tuple(literal + value for value in values) for literal, values in pieces]
    names = tuple(PREFIX + "".join(choice) + tail for choice in itertools.product(*options))
    if not names or len(names) > MAX_EXPANSIONS or len(names) != len(set(names)):
        reject("template expansion is empty, excessive, or generates duplicate names")
    return names


def literal(raw: bytes, page: int, section: str, declaration: str) -> dict[str, object]:
    text, _, name, _ = anchored(raw, page, section, declaration, False)
    return {"document_name": name, "c_name": PREFIX + name, "declaration": text,
            "physical_page": page, "section": section}


def template(raw: bytes, page: int, section: str, declaration: str) -> dict[str, object]:
    text, _, name, _ = anchored(raw, page, section, declaration, True)
    return {"declaration": text, "document_name": name, "c_names": list(expand_template_name(name)),
            "physical_page": page, "section": section}


def artifact_rules(raw: bytes) -> dict[str, object]:
    formal_pages(raw)
    return {"source_pages": [{"physical_page": page, "section": next(section for check, section, _, _ in ANCHORS if check == page),
                              "anchors": [label for check, _, label, _ in ANCHORS if check == page]}
                             for page in (32, 33, 34, 163)],
            "c_binding_prefix": {"document_command_prefix": "", "c_command_prefix": PREFIX,
                                 "applies_only_to": "bare-source-anchored-command-names"},
            "formal_template_grammar": {"digit_choices": list(DIGITS), "type_descriptors": [{"suffix": key, "type": value} for key, value in DESCRIPTORS],
                                        "dimension_choices": list(DIMENSIONS), "max_expansions": MAX_EXPANSIONS},
            "literal_examples": [literal(raw, 33, "2.2", value) for value in LITERALS],
            "template_examples": [template(raw, 33, "2.2", UNIFORM)],
            "rejections": ["unexpanded-template", "guessed-or-reordered-suffix", "duplicate-generated-name", "ambiguous-or-unanchored-declaration", "registry-header-lower-profile-extension-glsl-compatibility"]}
