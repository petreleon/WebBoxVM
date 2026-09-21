# F03.4.2.4 — Aggregate the no-claim provenance receipt

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.4.2.4
Depends: F03.4.2.1, F03.4.2.2, F03.4.2.3
Evidence: [receipt](evidence.md)

## Outcome

One small self-hashed receipt joins the sealed role-aware identities, raw registry facts, citation-only
Docs map, and root-wide VCTS diagnostic. It keeps raw technical facts separate from a reviewed Vulkan
core matrix and records every unresolved obligation as blocked.

## Starting points

- [source-channel boundary](../01-source-channel-boundary/README.md)
- [raw Docs provenance](../02-raw-docs-provenance/README.md)
- [VCTS diagnostics](../03-full-suite-diagnostics/README.md)
- [registry inventory](../../01-registry-inventory/README.md)

## Checklist

- [x] Bind the active source-contract and lock identities to the registry count, order, and raw-row digest.
- [x] Bind the Docs-map and VCTS-diagnostic receipt identities without treating either as a matrix import.
- [x] Require missing or duplicate raw rows, stale pins, mixed receipts, and scope relabeling to fail closed.
- [x] Keep all raw facts blocked, all claims false, CTS executions zero, and matrix-row count explicitly zero.
- [x] Run focused positive and hostile checks plus the required local gates; preserve unavailable external prerequisites as blocked.
- [x] Attach the aggregate receipt without marking F03.4 complete or claiming guest, browser, CTS, certification, or performance evidence.

## Verification

The aggregate makes planning provenance auditable only. F03.4 remains open until a later reviewed
semantic-core matrix assigns concrete owners and independent test plans.
