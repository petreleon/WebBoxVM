# F03.2.2.3 — Extract raw state and lifecycle facts

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.2.2.3
Depends: F03.2.2.1
Evidence: pending

## Outcome

A separate raw inventory records admitted non-limit/non-format state variables and explicitly specified
object lifecycle transitions. It keeps table facts distinct from prose-derived transitions and does not
infer behavior from a registry or implementation.

## Starting points

- [verified normative-PDF cache](../01-normative-pdf-cache/README.md)
- [source-authority boundary](../../01-source-authority/README.md)
- [limit and format sibling](../../03-limit-format-shader-inventory/README.md)

## Checklist

- [ ] Read state variables from the normative state tables and retain table and row positions in the raw
  condition while preserving the F03.2.1 PDF locator syntax.
- [ ] Record only explicit lifecycle or state-transition rules with their triggering command and source
  section; flag no inferred transition as a fact.
- [ ] Route numeric limits and format properties to F03.2.3 instead of duplicating or silently omitting
  them here.
- [ ] Exclude compatibility-only state, extensions, GLSL semantics, and implementation observations.
- [ ] Bind ordered table and section coverage to fail on a missing, duplicate, reordered, or cross-profile
  raw fact.
- [ ] Add focused table/prose hostile tests and attach a no-claim receipt.

## Verification

Chapter 23 lists queryable state but is not itself a complete transition grammar. A reviewable coverage
manifest is required before an extractor may call its raw state/lifecycle set complete.
