# F03.3.2.2.4 — Extract state and execution command slices

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.3.2.2.4
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: pending

## Outcome

Separate bounded inventories extract explicit state/lifecycle and non-object command facts. They retain a
triggering command and table or section anchor, without inferring behavior from a renderer or registry.

## Starting points

- [domain classification](../01-command-domain-classification/README.md)
- [declaration grammar](../02-template-declaration-grammar/README.md)
- [verified normative-PDF cache](../../01-normative-pdf-cache/README.md)

## Checklist

- [ ] [F03.3.2.2.4.1 — Extract context state and lifecycle facts](01-context-state-lifecycle/README.md)
- [ ] [F03.3.2.2.4.2 — Extract draw and raster commands](02-draw-raster-commands/README.md)
- [ ] [F03.3.2.2.4.3 — Extract pixel-transfer commands](03-pixel-transfer-commands/README.md)
- [ ] [F03.3.2.2.4.4 — Extract debug and special query commands](04-debug-special-queries/README.md)

## Verification

Every child remains raw-only and `matrix-incomplete`; it does not prove state behavior, ordering,
guest/browser execution, support, conformance, certification, or performance.
