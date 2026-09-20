# F03.3.2.5 — Handoff raw facts and guard matrix import

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.3.2.5
Depends: F03.3.2.2, F03.3.2.3, F03.3.2.4
Evidence: pending

## Outcome

The two admitted raw inventories and the unavailable-source ledger form one self-consistent handoff for
F03.3.3. A fail-closed import guard proves that source facts cannot masquerade as Matrix v2 rows before
real implementation owners and independent full-suite reference obligations exist.

## Starting points

- [command/object/state raw inventory](../02-command-object-state-raw-inventory/README.md)
- [limit/format raw inventory](../03-limit-format-raw-inventory/README.md)
- [unavailable language and extension ledger](../04-unavailable-language-extension-ledger/README.md)
- [shader and CTS obligations](../../03-shader-and-cts-obligations/README.md)

## Checklist

- [ ] Aggregate exact source identity, ordered raw counts, hashes, scopes, and unavailable-class blockers.
- [ ] Reject a missing raw family, stale cache identity, cross-profile source, duplicate fact, reordered
  handoff, or altered unavailable decision.
- [ ] Refuse Matrix v2 emission while a concrete implementation owner or independent full-suite reference
  obligation is absent.
- [ ] Keep raw facts and unavailable classes visible with `matrix-incomplete`, zero CTS executions, and no
  qualification claims.
- [ ] Hand the complete raw set to F03.3.3 without converting an unavailable class into a substituted
  shader, precision, or extension requirement.
- [ ] Add focused handoff and import-guard checks, then attach a no-claim receipt.

## Verification

The handoff is deliberately not a `matrix_contract_v2.py` document. F03.3.3 must first record genuine
ownership and independent reference obligations; a later importer may then attribute only eligible facts.
