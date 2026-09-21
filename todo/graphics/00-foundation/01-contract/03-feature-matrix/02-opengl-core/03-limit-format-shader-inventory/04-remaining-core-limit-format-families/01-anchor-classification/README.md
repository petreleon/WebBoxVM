# F03.2.3.4.1 — Classify remaining anchors

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.3.4.1
Depends: F03.2.1, F03.2.2.1, F03.2.3.1, F03.2.3.2
Evidence: pending

## Outcome

One self-hashed, closed classification catalog assigns every remaining candidate anchor to `covered`,
`eligible-unreviewed`, `route-to-state`, `shader-unadmitted`, `extension-unadmitted`, or `out-of-domain`.

## Starting points

- [reviewed table slice](../../01-limit-format-raw-inventory/README.md)
- [unadmitted source ledger](../../02-unadmitted-shader-extension-ledger/README.md)
- [normative PDF cache](../../../02-command-object-state-inventory/01-normative-pdf-cache/README.md)

## Checklist

- [ ] Bind the exact cache, source authority, reviewed table artifact, and unavailable-source ledger.
- [ ] Catalog tables 23.56–23.70 (pages 660–674) and 23.72–23.74 (pages 676–678) at row/column granularity.
- [ ] Catalog chapter-local format and constraint families without turning headings into automatic source-class decisions.
- [ ] Give every candidate anchor exactly one route and reject gaps, duplicates, reordered rows, and catch-all labels.
- [ ] Preserve source page, section, table/row, source order, and explicit reason for every route.
- [ ] Emit no facts or Matrix/CTS/claim promotion; attach a no-claim receipt.

## Verification

The catalog classifies review work only. It cannot infer an API limit, shader authority, extension support,
format behavior, guest/browser behavior, conformance, certification, or performance.
