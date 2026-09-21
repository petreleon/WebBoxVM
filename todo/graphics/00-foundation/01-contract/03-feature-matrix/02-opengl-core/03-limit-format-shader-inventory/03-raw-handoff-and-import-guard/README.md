# F03.2.3.3 — Handoff raw facts and guard Matrix import

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.2.3.3
Depends: F03.2.3.1, F03.2.3.2, F03.2.3.4
Evidence: pending

## Outcome

The admitted limit/format raw facts and unresolved shader/extension decisions form one reviewable handoff
for F03.2.4. A fail-closed guard prevents either source data or policy state from masquerading as a Matrix
v2 row before its owner and independent full-suite reference obligation exist.

## Starting points

- [limit/format raw inventory](../01-limit-format-raw-inventory/README.md)
- [unadmitted source ledger](../02-unadmitted-shader-extension-ledger/README.md)
- [remaining core limit/format families](../04-remaining-core-limit-format-families/README.md)
- [ownership and reference obligations](../../04-ownership-and-cts-obligations/README.md)

## Checklist

- [ ] Aggregate exact source identity, cache identity, ordered raw counts, hashes, coverage scopes, and
  unresolved source-policy decisions; refuse completion while the reviewed table slice leaves core
  limit/format families unclassified.
- [ ] Reject a missing source family, stale cache identity, cross-profile data, duplicate fact, reordered
  handoff, or a changed unavailable decision.
- [ ] Refuse Matrix v2 emission until F03.2.4.1 records a real owner and independent full-suite reference
  obligation for every admitted raw fact, then F03.2.4.2 performs the import.
- [ ] Preserve shader and extension classes as unavailable rather than turning their absence into a
  supported, optional, or untracked exclusion.
- [ ] Keep raw facts blocked with zero CTS executions and `matrix-incomplete`; attach a no-claim receipt.
- [ ] Provide the precise F03.2.4 handoff without asserting guest, browser, certification, or performance
  behavior.

## Verification

The handoff is deliberately not a `matrix_contract_v2.py` document. It records raw source state and
admission gaps only; owners, test obligations, matrix rows, and all runtime evidence remain downstream.
