# F03.4.2 — Attach Docs provenance and CTS diagnostics

[Parent task](../README.md)

Task: F03.4.2
Depends: F03.4.1, F02.5.4.2
Evidence: pending

Prerequisite lists: [the registry inventory](../01-registry-inventory/README.md) and [the active
role-aware gate](../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md).

## Outcome

One later mapping attaches raw pinned Docs locators for normative provenance and the immutable full
vk-default category report for diagnostics. It preserves each channel's limits and cannot make a local
core-only CTS selector or conformance claim.

## Starting points

- [registry inventory](../01-registry-inventory/README.md)
- [active role-aware gate](../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/02-f03-gate-and-matrix-binding/README.md)

## Checklist

- [ ] Bind raw Docs citations by revision, path, digest, license/attribution, and semantic locator.
- [ ] Map each matrix row to the unfiltered canonical CTS taxonomy without filtering or scope relabeling.
- [ ] Keep generated Docs, inferred authority, and unreviewed mappings out of all source roles.
- [ ] Require independent implementation, guest/native reference, CTS, and browser evidence for release claims.
- [ ] Reject missing provenance, mixed pins, local selectors, and false release effects with tests and a receipt.

## Verification

- Passing this mapping permits only auditable planning. It does not prove guest-visible compatibility,
  Khronos certification, or near-native browser performance.
