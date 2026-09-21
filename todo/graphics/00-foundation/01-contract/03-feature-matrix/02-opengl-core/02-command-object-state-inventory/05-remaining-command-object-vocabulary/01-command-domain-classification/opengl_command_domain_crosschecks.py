"""Independent bounded provenance crosschecks for F03.2.2.5.1."""

from __future__ import annotations

import re

CREATE_TERM = re.compile(r"\bCreate[A-Za-z0-9]+\b")
LOCATOR = re.compile(r"^opengl46-core-pdf-v1:page=([1-9][0-9]*);section=([1-9][0-9]*(?:\.[1-9][0-9]*)*)$")
SECTION = re.compile(r"^[1-9][0-9]*(?:\.[1-9][0-9]*)*$")

# These are actual, bounded Create* index entries on physical p. 808, not API facts.
INDEX_WITNESSES = (
    ("CreateBuffers", "buffer", "buffer-commands", 808),
    ("CreateFramebuffers", "framebuffer-renderbuffer", "framebuffer-renderbuffer-commands", 808),
    ("CreateProgram", "program-pipeline", "program-pipeline-commands", 808),
    ("CreateProgramPipelines", "program-pipeline", "program-pipeline-commands", 808),
    ("CreateQueries", "event-query-sync", "generic-object-sync", 808),
    ("CreateRenderbuffers", "framebuffer-renderbuffer", "framebuffer-renderbuffer-commands", 808),
    ("CreateSamplers", "texture-sampler", "texture-sampler-commands", 808),
    ("CreateShader", "program-pipeline", "program-pipeline-commands", 808),
    ("CreateShaderProgramv", "program-pipeline", "program-pipeline-commands", 808),
    ("CreateTextures", "texture-sampler", "texture-sampler-commands", 808),
    ("CreateTransformFeedbacks", "transform-feedback", "vertex-transform-feedback-commands", 808),
    ("CreateVertexArrays", "vertex-array", "vertex-transform-feedback-commands", 808),
)

# fact id, expected remaining-family owner, actual F03.2.2.2 locator, page, section
BASELINE_BINDINGS = (
    ("command:glCreateBuffers", "buffer", "opengl46-core-pdf-v1:page=81;section=6.1", 81, "6.1"),
    ("command:glCreateShader", "program-pipeline", "opengl46-core-pdf-v1:page=110;section=7.1", 110, "7.1"),
    ("command:glCreateProgram", "program-pipeline", "opengl46-core-pdf-v1:page=116;section=7.3", 116, "7.3"),
    ("command:glCreateProgramPipelines", "program-pipeline", "opengl46-core-pdf-v1:page=145;section=7.4", 145, "7.4"),
    ("command:glCreateTextures", "texture-sampler", "opengl46-core-pdf-v1:page=204;section=8.1", 204, "8.1"),
    ("command:glCreateSamplers", "texture-sampler", "opengl46-core-pdf-v1:page=206;section=8.2", 206, "8.2"),
    ("command:glCreateRenderbuffers", "framebuffer-renderbuffer", "opengl46-core-pdf-v1:page=335;section=9.2.4", 335, "9.2.4"),
    ("command:glCreateFramebuffers", "framebuffer-renderbuffer", "opengl46-core-pdf-v1:page=324;section=9.2", 324, "9.2"),
    ("command:glCreateVertexArrays", "vertex-array", "opengl46-core-pdf-v1:page=373;section=10.3.1", 373, "10.3.1"),
    ("command:glCreateTransformFeedbacks", "transform-feedback", "opengl46-core-pdf-v1:page=466;section=13.3.1", 466, "13.3.1"),
    ("command:glCreateQueries", "event-query-sync", "opengl46-core-pdf-v1:page=66;section=4.2.2", 66, "4.2.2"),
    ("command:glFenceSync", "event-query-sync", "opengl46-core-pdf-v1:page=58;section=4.1", 58, "4.1"),
)


def section_key(value: str):
    if not isinstance(value, str) or not SECTION.fullmatch(value):
        return None
    return tuple(int(part) for part in value.split("."))


def scope_contains(scope: str, section: str) -> bool:
    key = section_key(section)
    if not isinstance(scope, str) or key is None:
        return False
    for part in scope.split(";"):
        bounds = part.split("-", 1)
        left, right = section_key(bounds[0]), section_key(bounds[-1])
        if left is not None and right is not None and left <= key <= right:
            return True
    return False


def binding(anchor: object, reject):
    if not isinstance(anchor, dict) or set(anchor) != {"fact_id", "source_locator", "physical_page"}:
        reject("baseline anchor shape is not exact")
    fact, locator, page = anchor["fact_id"], anchor["source_locator"], anchor["physical_page"]
    match = LOCATOR.fullmatch(locator) if isinstance(locator, str) else None
    if not isinstance(fact, str) or type(page) is not int or match is None or int(match.group(1)) != page:
        reject("baseline anchor has an invalid locator or page")
    return fact, locator, page, match.group(2)


def baseline_assignments(baseline: object, rules, reject) -> dict[str, list[dict[str, object]]]:
    if not isinstance(baseline, dict):
        reject("baseline binding is not an object")
    anchors = baseline.get("direct_creation_anchors")
    if not isinstance(anchors, list) or len(anchors) != len(BASELINE_BINDINGS):
        reject("baseline source anchors are not an exact list")
    actual = tuple((*binding(item, reject)[:1], owner, *binding(item, reject)[1:])
                   for item, (_, owner, *_rest) in zip(anchors, BASELINE_BINDINGS))
    if (actual != BASELINE_BINDINGS or baseline.get("direct_creation_command_ids")
            != [item[0] for item in BASELINE_BINDINGS]):
        reject("baseline source locators, pages, sections, or fact IDs are not exact")
    owners = {fact: owner for fact, owner, *_ in BASELINE_BINDINGS}
    if rules.BASELINE_OWNER != owners:
        reject("hard-coded baseline owner map does not match the source-bound assignment")
    families = {row[0]: row for row in rules.COMMAND_FAMILIES}
    assigned = {identifier: [] for identifier in families}
    for fact, owner, locator, page, section in BASELINE_BINDINGS:
        family = families.get(owner)
        if family is None or not scope_contains(family[1], section):
            reject("baseline owner anchor falls outside the declared source scope")
        assigned[owner].append({"fact_id": fact, "source_locator": locator,
                                "physical_page": page, "section": section})
    return assigned


def index_witnesses(chunks: list[str], pages: tuple[int, ...], rules, reject) -> list[dict[str, object]]:
    actual = tuple(sorted((term, pages[offset]) for offset, chunk in enumerate(chunks)
                          for term in set(CREATE_TERM.findall(rules.normalized(chunk)))))
    expected = tuple((term, page) for term, _family, _route, page in INDEX_WITNESSES)
    if actual != expected:
        reject("bounded Create* index witness set has an unknown, missing, or unaccounted entry")
    family_routes = {identifier: route for identifier, _scope, _section, _page, _decl, route in rules.COMMAND_FAMILIES}
    if any(family_routes.get(family) != route for _term, family, route, _page in INDEX_WITNESSES):
        reject("index witness route is not the declared source-family route")
    return [{"term": term, "physical_page": page, "expected_family_id": family, "expected_route": route}
            for term, family, route, page in INDEX_WITNESSES]
