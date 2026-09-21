"""F03.2.2.3.1 reviewed state-table and lifecycle anchors in the sealed core PDF."""

import subprocess

PROFILE = "opengl-4.6-core"
PREFIX = "opengl46-core-pdf-v1:page="
REBIND = ("BindBuffer may also be used to bind an existing buffer object. If the bind is "
          "successful no change is made to the state of the newly bound buffer object, and any "
          "previous binding to target is broken.")
DELETE = ("If a buffer object is deleted while it is bound, all bindings to that object in "
          "the current context (i.e. in the thread that called DeleteBuffers) are reset to zero. "
          "Bindings to that buffer in other contexts are not affected, and the deleted buffer "
          "may continue to be used at any places it remains bound or attached, as described "
          "in section 5.1.")
TABLE = "Table 23.5: Vertex Array Data (not in Vertex Array objects)"
ROW = "ARRAY BUFFER BINDING Z+ GetIntegerv 0 Current buffer binding 6"
SCOPES = [
    {"physical_page": 84, "section": "6.1", "kind": "explicit-prose-transitions",
     "anchor_ids": ["buffer-existing-rebind", "buffer-delete-current-context"]},
    {"physical_page": 609, "section": "23", "kind": "state-table-row",
     "table": "23.5", "reviewed_rows": [1], "anchor_ids": ["array-buffer-binding-state"]},
]
EXCLUSIONS = [
    "Remaining table 23.5 rows and all other chapter 23 tables are unreviewed here.",
    "Remaining section 6.1 prose and all other lifecycle rules are unreviewed here.",
    "Numeric limits and format properties belong to F03.2.3; none selected here.",
    "Compatibility profile, extensions, GLSL semantics and implementation observations excluded.",
    "No complete OpenGL state universe, transition grammar or coverage inference.",
]


class AnchorError(ValueError):
    """A reviewed anchor is absent, ambiguous or out of source order."""


def extract(raw):
    try:
        result = subprocess.run(["pdftotext", "-layout", "-", "-"], input=raw,
                                capture_output=True, check=False)
        pages = result.stdout.decode("utf-8").split("\f")
    except (OSError, UnicodeDecodeError) as error:
        raise AnchorError(f"pdftotext could not extract the verified PDF: {error}") from error
    if result.returncode or len(pages) != 852 or pages[-1].strip():
        raise AnchorError("extraction does not contain 851 physical PDF pages")
    return {page: " ".join(pages[page - 1].split()) for page in (84, 609)}


def unique(text, anchor):
    if text.count(anchor) != 1:
        raise AnchorError("reviewed anchor is missing or ambiguous")
    return text.index(anchor)


def facts_from_pages(pages):
    prose, table = pages.get(84, ""), pages.get(609, "")
    heading = unique(prose, "6.1. CREATING AND BINDING BUFFER OBJECTS")
    rebind, delete = unique(prose, REBIND), unique(prose, DELETE)
    next_section = unique(prose, "6.1.1 Binding Buffer Objects to Indexed Targets")
    if not heading < rebind < delete < next_section:
        raise AnchorError("reviewed prose anchors are reordered or outside section 6.1")
    if unique(table, TABLE) >= unique(table, ROW):
        raise AnchorError("state row precedes its reviewed table")
    # The next row independently fixes row 1's position and catches row substitution.
    if unique(table, ROW) >= unique(table, "DRAW INDIRECT BUFFER BINDING"):
        raise AnchorError("reviewed table rows are reordered")
    return [
        {"id": "buffer-existing-rebind", "kind": "explicit-lifecycle-transition",
         "source_locator": PREFIX + "84;section=6.1", "source_order": 1,
         "trigger": "BindBuffer", "raw_condition": "Successful bind of an existing buffer object.",
         "raw_fact": "Newly bound object's state unchanged; previous target binding broken.",
         "anchor_text": REBIND},
        {"id": "buffer-delete-current-context", "kind": "explicit-lifecycle-transition",
         "source_locator": PREFIX + "84;section=6.1", "source_order": 2,
         "trigger": "DeleteBuffers", "raw_condition": "Deletion of a bound buffer object.",
         "raw_fact": "Current-context bindings reset to zero; other contexts unaffected; see section 5.1.",
         "anchor_text": DELETE},
        {"id": "array-buffer-binding-state", "kind": "state-table-fact",
         "source_locator": PREFIX + "609;section=23", "source_order": 3,
         "table": "23.5", "table_row": 1, "state": "ARRAY_BUFFER_BINDING",
         "query": "GetIntegerv", "initial_value": "0", "referenced_section": "6",
         "raw_condition": "Table 23.5 row 1: vertex array data outside vertex array objects.",
         "anchor_text": ROW},
    ]


def coverage():
    return {"scope": "three-reviewed-facts-only", "complete_within_selected_anchors": True,
            "full_state_inventory_complete": False, "ordered_sources": SCOPES,
            "exclusions": EXCLUSIONS}
