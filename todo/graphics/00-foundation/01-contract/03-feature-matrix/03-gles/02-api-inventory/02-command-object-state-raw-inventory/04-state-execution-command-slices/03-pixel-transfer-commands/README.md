# F03.3.2.2.4.3 — Extract pixel-transfer commands

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F03.3.2.2.4.3
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: pending

## Outcome

A bounded raw inventory records formal pixel-transfer and copy command declarations, while format conversion,
packing, image layout, readback behavior, and state remain separately reviewed source domains.

## Starting points

- [state/execution slice list](../README.md)
- [declaration grammar](../../02-template-declaration-grammar/README.md)
- [limit/format inventory](../../../03-limit-format-raw-inventory/README.md)

## Checklist

- [ ] Bind exact authority, cache, domain map, and declaration grammar identities.
- [ ] Extract only formal declarations with physical page, section, and source-order anchors.
- [ ] Route format conversion, packing, layout, state, limits, and readback semantics out of command facts.
- [ ] Reject extensions, desktop/lower profiles, registry data, guessed templates, and ESSL semantics.
- [ ] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not prove pixel correctness, readback behavior, guest/browser rendering, support,
conformance, certification, or performance.
