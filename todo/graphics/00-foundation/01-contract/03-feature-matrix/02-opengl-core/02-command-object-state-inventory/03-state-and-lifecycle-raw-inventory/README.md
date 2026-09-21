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

- [ ] [F03.2.2.3.1 — Buffer-binding lifecycle raw slice](01-buffer-binding-lifecycle-raw-slice/README.md)
- [ ] [F03.2.2.3.2 — Remaining state and lifecycle families](02-other-state-lifecycle-families/README.md)
- [ ] [F03.2.2.3.3 — Aggregate coverage and raw handoff guard](03-aggregate-raw-handoff-guard/README.md)

## Verification

Chapter 23 lists queryable state but is not itself a complete transition grammar. A reviewable coverage
manifest is required before an extractor may call its raw state/lifecycle set complete.

F03.2.2.3.1 is limited to three reviewed buffer-binding facts. It explicitly records `complete=false`
for the state/lifecycle universe and cannot complete this parent. The remaining family inventory,
ordered source coverage, exclusions and aggregate no-claim handoff remain required below.
