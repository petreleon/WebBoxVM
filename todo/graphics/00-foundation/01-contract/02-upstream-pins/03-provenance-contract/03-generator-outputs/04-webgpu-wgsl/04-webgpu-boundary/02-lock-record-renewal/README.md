# F02.3.3.4.4.2 — Renew records after the WebIDL inventory change

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.3.3.4.4.2
Depends: F02.3.3.4.4.1, F02.3.1
Evidence: pending

Prerequisite lists: [WebIDL source admission](../01-webidl-source-admission/README.md),
[provenance contract](../../../../01-provenance-record/README.md), and
[F06 reproducibility](../../../../../../../02-reproducibility/README.md).

## Outcome

All existing F02.3 records and F06 reproducibility metadata bind the new raw
inventory-lock identity without changing their own input identities or claiming
a WebGPU feature.

## Starting points

- [canonical inventory lock](../../../../../01-input-inventory/inventory.lock)
- [ABI sidecar records](../../../../02-abi-fixtures/records/)
- [F06 generated records](../../../../../../../../00-foundation/02-reproducibility/03-file-layout/03-generation-and-checker/README.md)
- [prior renewal receipt](../../02-record-renewal/evidence.md)

## Checklist

- [ ] Enumerate every committed F02.3 and F06 consumer of the inventory-lock identity.
- [ ] Renew only the derived lock identity and required reproducibility output bytes.
- [ ] Prove every renewed record rejects the previous lock and preserves non-derived fields.
- [ ] Re-run the relevant provenance, reproducibility, inventory, and source-limit suites locally.

## Verification

- No sidecar retains the prior inventory-lock digest.
- An old-lock fixture is rejected without network access; unchanged input references remain valid.

## Scope limit

Renewal proves record provenance only. It does not create a WebIDL parser, binding, browser API,
guest-visible protocol, renderer, compatibility result, or performance measurement.
