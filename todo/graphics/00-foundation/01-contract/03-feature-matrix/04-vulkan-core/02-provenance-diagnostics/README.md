# F03.4.2 — Attach Docs provenance and CTS diagnostics

[Parent task](../README.md)

Task: F03.4.2
Depends: F03.4.1, F02.5.4.2
Evidence: [aggregate receipt](04-aggregate-no-claim-receipt/evidence.md)

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

- [x] [F03.4.2.1 — Seal provenance channels and the raw-to-matrix boundary](01-source-channel-boundary/README.md)
- [x] [F03.4.2.2 — Pin citation-only raw Docs locators](02-raw-docs-provenance/README.md)
- [x] [F03.4.2.3 — Bind unfiltered VCTS diagnostics](03-full-suite-diagnostics/README.md)
- [x] [F03.4.2.4 — Aggregate the no-claim provenance receipt](04-aggregate-no-claim-receipt/README.md)

## Verification

- Passing this mapping permits only auditable planning. It does not prove guest-visible compatibility,
  Khronos certification, or near-native browser performance.

## Split rationale

The sealed F02 role contract admits a normative Docs root and an unfiltered full-suite root, not a
semantic Docs closure or a core-only CTS selector. Separating the channel boundary, raw citation map,
root-wide diagnostic report, and aggregate prevents generated Docs, an inferred row-to-case match, or a
local filter from becoming source authority. A later semantic-core review remains responsible for actual
matrix rows, owners, and independent test plans.
