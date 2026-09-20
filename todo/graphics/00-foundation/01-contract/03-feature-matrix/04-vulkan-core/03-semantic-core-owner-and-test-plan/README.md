# F03.4.3 — Review semantic core scope, owners, and test plans

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.4.3
Depends: F03.4.2.4, F02.5.4.2
Evidence: pending

## Outcome

A reviewed Vulkan 1.4 public-core matrix derives only semantically justified mandatory rows from the
raw inventory and provenance record. Every admitted row has an implementation owner and an independent
native, guest, or full-suite test obligation; unresolved facts remain blocked and visible.

## Starting points

- [provenance and diagnostics aggregate](../02-provenance-diagnostics/04-aggregate-no-claim-receipt/README.md)
- [raw registry inventory](../01-registry-inventory/README.md)
- [F03 v2 matrix contract](../../01-profile-scope/matrix_contract_v2.py)
- [active source gate](../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md)

## Checklist

- [ ] Review each proposed mandatory public-core row against its raw Docs provenance; keep structural, extension, WSI, video, and unknown facts distinct.
- [ ] Import only rows with the exact normative-root and full-suite-root bindings required by the active matrix contract.
- [ ] Assign a concrete downstream implementation owner and independent test obligation, or retain an explicit blocked reason.
- [ ] Do not substitute the auxiliary registry, a generated Docs artifact, a local CTS selector, or bounded VirGL evidence for reviewed semantics.
- [ ] Reject missing/duplicate rows, stale provenance, empty owners, generic tests, and unsupported rows marked supported.
- [ ] Attach a no-claim coverage receipt; defer profile-bound check registration to F05.2.

## Verification

This creates blocked planning obligations, not implementation evidence. It does not run CTS or prove a
guest API, browser path, conformance, certification, support, or near-native performance.
