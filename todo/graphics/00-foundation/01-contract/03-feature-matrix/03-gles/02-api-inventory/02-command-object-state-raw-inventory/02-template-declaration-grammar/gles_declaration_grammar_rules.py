"""Finite, source-anchored GLES 3.2 declaration spelling rules."""

from __future__ import annotations

import itertools
import re
import subprocess

PROFILE, PAGES, PREFIX = "gles-3.2", 601, "gl"
LITERAL = re.compile(r"[A-Z][A-Za-z0-9]*\Z")
GROUPS = (
    ("1234", ("1", "2", "3", "4")), ("234", ("2", "3", "4")), ("if", ("i", "f")),
    ("if ui", ("i", "f", "ui")), ("i ui", ("i", "ui")), ("f", ("f",)),
    ("2x3,3x2,2x4,4x2,3x4,4x3", ("2x3", "3x2", "2x4", "4x2", "3x4", "4x3")),
)
BASE_ANCHORS = (
    (26, "2.1", "Finally, command names, constants, and types are prefixed in the C language binding to OpenGL ES (by gl, GL_, and GL, respectively), to reduce name clashes with other packages. The prefixes are omitted in this document for clarity."),
    (26, "2.2", "2.2 Command Syntax"), (27, "2.2", "Type Descriptor Corresponding GL Type"),
    (27, "2.2", "void Uniform{1234}{if}( int location, T value );"), (28, "2.2", "indicates the eight declarations"),
)
FORMAL_TEMPLATES = (
    (27, "2.2", "Uniform{1234}{if}"),
    (126, "7.6.1", "Uniform{1234}{if ui}"), (126, "7.6.1", "Uniform{1234}{if ui}v"),
    (126, "7.6.1", "UniformMatrix{234}fv"), (126, "7.6.1", "UniformMatrix{2x3,3x2,2x4,4x2,3x4,4x3}fv"),
    (128, "7.6.1", "ProgramUniform{1234}{if}"), (128, "7.6.1", "ProgramUniform{1234}{if}v"),
    (128, "7.6.1", "ProgramUniform{1234}ui"), (128, "7.6.1", "ProgramUniform{1234}uiv"),
    (129, "7.6.1", "ProgramUniformMatrix{234}{f}v"), (129, "7.6.1", "ProgramUniformMatrix{2x3,3x2,2x4,4x2,3x4,4x3}{f}v"),
    (160, "8.2", "SamplerParameter{if}"), (160, "8.2", "SamplerParameter{if}v"), (160, "8.2", "SamplerParameterI{i ui}v"),
    (162, "8.3", "GetSamplerParameter{if}v"), (162, "8.3", "GetSamplerParameterI{i ui}v"),
    (206, "8.10", "TexParameter{if}"), (206, "8.10", "TexParameter{if}v"), (206, "8.10", "TexParameterI{i ui}v"),
    (209, "8.11.2", "GetTexParameter{if}v"), (209, "8.11.2", "GetTexParameterI{i ui}v"),
    (210, "8.11.3", "GetTexLevelParameter{if}v"),
    (283, "10.2.1", "VertexAttrib{1234}f"), (283, "10.2.1", "VertexAttrib{1234}fv"),
    (283, "10.2.1", "VertexAttribI4{i ui}"), (283, "10.2.1", "VertexAttribI4{i ui}v"),
    (420, "15.2.3.1", "ClearBuffer{if ui}v"),
)
SECTION_WITNESSES = {
    "2.1": (25, "2.1. OPENGL ES FUNDAMENTALS"), "2.2": (26, "2.2 Command Syntax"),
    "7.6.1": (126, "7.6.1 Loading Uniform Variables In The Default Uniform Block"),
    "8.2": (158, "8.2 Sampler Objects"), "8.3": (161, "8.3 Sampler Object Queries"),
    "8.10": (206, "8.10 Texture Parameters"), "8.11.2": (209, "8.11.2 Texture Parameter Queries"),
    "8.11.3": (210, "8.11.3 Texture Level Parameter Queries"),
    "10.2.1": (283, "10.2.1 Current Generic Attributes"), "15.2.3.1": (420, "15.2.3.1 Clearing Individual Buffers"),
}
FORMAL_NAMES = frozenset(item[2] for item in FORMAL_TEMPLATES)


class GrammarError(ValueError):
    """A purported declaration spelling is outside the sealed formal grammar."""


def reject(message: str) -> None:
    raise GrammarError(message)


def compact(value: str) -> str:
    return " ".join(value.replace("\f", " ").split())


def page_text(raw: bytes, page: int) -> str:
    try:
        result = subprocess.run(["pdftotext", "-f", str(page), "-l", str(page), "-raw", "-", "-"], input=raw, capture_output=True, check=False)
        value = result.stdout.decode("utf-8")
    except (OSError, UnicodeDecodeError) as error:
        reject(f"cannot read sealed declaration-grammar page: {error}")
    if result.returncode != 0:
        reject("cannot read sealed declaration-grammar page")
    return compact(value)


def c_command_name(value: object) -> str:
    if not isinstance(value, str) or not LITERAL.fullmatch(value) or value.startswith(("gl", "GL")):
        reject("literal declaration name is not an unprefixed C-spec command spelling")
    return PREFIX + value


def expand_template_name(value: object) -> list[str]:
    if not isinstance(value, str) or value not in FORMAL_NAMES:
        reject("template is not one of the sealed PDF formal declaration forms")
    choices, cursor = [], 0
    for match in re.finditer(r"\{([^{}]+)\}", value):
        literal, group = value[cursor:match.start()], dict(GROUPS).get(match.group(1))
        if not re.fullmatch(r"[A-Za-z0-9]*", literal) or group is None:
            reject("template has an unanchored, malformed, or guessed choice group")
        choices.append(tuple(literal + choice for choice in group)); cursor = match.end()
    tail = value[cursor:]
    if not choices or not re.fullmatch(r"[A-Za-z0-9]*", tail):
        reject("template is unexpanded or malformed")
    names = [PREFIX + "".join(parts) + tail for parts in itertools.product(*choices)]
    if not names or len(names) > 64 or len(names) != len(set(names)):
        reject("template expansion is empty, excessive, or duplicate")
    return names


def template_catalog() -> list[dict[str, object]]:
    if len(FORMAL_NAMES) != len(FORMAL_TEMPLATES):
        reject("formal template catalog has duplicate source forms")
    return [{"formal_name": name, "physical_page": page, "section": section, "source_order": order,
             "expansion_count": len(expand_template_name(name))}
            for order, (page, section, name) in enumerate(FORMAL_TEMPLATES, 1)]


def check_anchors(raw: bytes, pages: int) -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    if pages != PAGES:
        reject("GLES declaration grammar has the wrong physical page count")
    texts: dict[int, str] = {}
    def text(page: int) -> str:
        texts.setdefault(page, page_text(raw, page)); return texts[page]
    def section(section: str) -> None:
        witness = SECTION_WITNESSES.get(section)
        if witness is None or witness[1] not in text(witness[0]):
            reject("GLES declaration-grammar section witness is missing or stale")
    rows = []
    for order, (page, heading, needle) in enumerate(BASE_ANCHORS, 1):
        section(heading)
        if needle not in text(page):
            reject("GLES declaration-grammar base anchor is missing or stale")
        rows.append({"physical_page": page, "section": heading, "text": needle, "source_order": order})
    catalog = template_catalog()
    for item in catalog:
        section(str(item["section"]))
        if f"void {item['formal_name']}(" not in text(int(item["physical_page"])):
            reject("GLES formal template declaration is missing or stale")
    return rows, catalog


def normalize(forms: object) -> list[str]:
    if not isinstance(forms, list):
        reject("declaration forms must be an ordered list")
    result: list[str] = []
    for form in forms:
        if not isinstance(form, tuple) or not form:
            reject("declaration form is malformed")
        if form[0] == "literal" and len(form) == 2:
            values = [c_command_name(form[1])]
        elif form[0] == "template" and len(form) == 2:
            values = expand_template_name(form[1])
        else:
            reject("unexpanded, guessed, registry, or non-command declaration form")
        if any(value in result for value in values):
            reject("normalized declaration spelling is duplicated")
        result.extend(values)
    return result
