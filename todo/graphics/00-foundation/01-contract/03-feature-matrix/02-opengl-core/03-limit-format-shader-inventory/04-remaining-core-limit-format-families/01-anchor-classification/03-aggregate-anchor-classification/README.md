# F03.2.3.4.1.3 — Aggregate the closed anchor classification

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F03.2.3.4.1.3
Depends: F03.2.3.4.1.1, F03.2.3.4.1.2
Evidence: [receipt](evidence.md)

## Outcome

One self-hashed receipt joins the final-table catalog and chapter-local manifest, then assigns every candidate
exactly one route: `covered`, `eligible-unreviewed`, `route-to-state`, `shader-unadmitted`,
`extension-unadmitted`, or `out-of-domain`.

## Starting points

- [final-table catalog](../01-final-table-anchors/README.md)
- [chapter-local manifest](../02-chapter-local-anchor-manifest/README.md)
- [parent anchor classification](../README.md)

## Checklist

- [x] Require exact child receipts, profile, cache, authority, reviewed-artifact, and ledger identities.
- [x] Bind every child candidate once with its immutable source locator and one fixed route/reason.
- [x] Route coverage to F03.2.3.4.2–F03.2.3.4.5 or an explicit non-admitted/state boundary.
- [x] Reject gaps, duplicates, overlaps, reordering, catch-all labels, stale child hashes, and promotions.
- [x] Keep all rows raw-only with zero Matrix rows, CTS runs, ownership, support, or performance fields.
- [x] Attach the aggregate receipt; only it may close F03.2.3.4.1.

## Verification

This receipt routes source-review work. It does not establish any API property or implementation behavior,
guest/browser result, conformance, certification, or performance measurement.
