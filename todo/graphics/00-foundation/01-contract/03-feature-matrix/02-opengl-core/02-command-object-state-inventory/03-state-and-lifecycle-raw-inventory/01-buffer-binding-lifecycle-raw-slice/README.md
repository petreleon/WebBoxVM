# F03.2.2.3.1 — Buffer-binding lifecycle raw slice

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.2.3.1
Depends: F03.2.2.1
Evidence: [receipt](evidence.md)

## Outcome

Three reviewed facts from the exact external OpenGL 4.6 core PDF: existing-buffer rebind and
current-context deletion transitions on physical page 84, section 6.1; initial/query state for
ARRAY_BUFFER_BINDING on physical page 609, table 23.5 row 1. Table facts remain distinct from prose.

## Starting points

- [Verified normative PDF cache](../../01-normative-pdf-cache/README.md)
- [OpenGL source authority](../../../01-source-authority/README.md)

## Checklist

- [x] Bind the exact F03.2.2.1 cache and F03.2.1 command-object-state authority through private imports.
- [x] Preserve physical pages, numeric section locators, triggering commands and table/row positions.
- [x] Verify unique ordered source anchors and a self-hashed inventory for the three reviewed facts.
- [x] Record `complete=false`, reviewed coverage and excluded remaining state/lifecycle families;
  route numeric limits/formats to F03.2.3 and exclude compatibility, extensions and GLSL semantics.
- [x] Reject missing, ambiguous, mixed, duplicate, reordered and promoted facts with focused checks.
- [x] Attach a no-claim receipt after local integration validation.

## Verification

Run `python3 opengl_state_raw_inventory_test.py` from this directory with
`WEBBOXVM_GRAPHICS_CACHE_ROOT` set to the verified external F02 cache root. Expected: eight tests,
three raw facts, zero Matrix rows and CTS executions, all support/conformance/performance claims false.
The bounded slice does not complete F03.2.2.3 or establish a complete transition grammar.
