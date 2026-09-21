# F03.2.2.3.2 — Remaining state and lifecycle families

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.2.3.2
Depends: F03.2.2.1
Evidence: pending

## Outcome

An explicit inventory and coverage decision for remaining admitted state-table and object-lifecycle
families, beyond the three buffer-binding facts in F03.2.2.3.1. Split this leaf into bounded family
children before implementing independent families; the parent outcome remains unchanged.

## Starting points

- [Verified normative PDF cache](../../01-normative-pdf-cache/README.md)
- [Buffer-binding slice](../01-buffer-binding-lifecycle-raw-slice/README.md)

## Checklist

- [ ] Enumerate admitted non-limit/non-format chapter 23 table rows and lifecycle sections, identifying
  remaining buffer, texture, vertex-array, framebuffer, renderbuffer, query, sync and other families.
- [ ] Split independent families into reviewable children with exact ordered source coverage.
- [ ] Retain physical PDF pages, numeric section locators and table/row positions for every table fact.
- [ ] Retain only explicit prose transitions with triggering commands; record all unreviewed rules.
- [ ] Route numeric limits and format properties to F03.2.3; exclude compatibility, extensions, GLSL
  semantics and implementation observations without silently discarding admitted state families.
- [ ] Verify missing, duplicate, reordered and cross-profile facts; attach aggregate family receipts.

## Verification

Use only the exact F03.2.2.1 cache and F03.2.1 command-object-state source class. Every family needs
source-based coverage; a regex hit list cannot establish completeness. No Matrix, owner/test, CTS,
support, conformance or performance claim is authorized by this raw source inventory.
