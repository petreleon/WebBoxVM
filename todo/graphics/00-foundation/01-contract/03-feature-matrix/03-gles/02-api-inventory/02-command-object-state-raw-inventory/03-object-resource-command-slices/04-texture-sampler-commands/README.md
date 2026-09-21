# F03.3.2.2.3.4 — Extract texture and sampler commands

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F03.3.2.2.3.4
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: pending

## Outcome

A bounded raw inventory records formal texture and sampler command declarations, separately routing image
formats, completeness, sampling behavior, pixel layout, and state to their dedicated review domains.

## Starting points

- [object/resource slice list](../README.md)
- [declaration grammar](../../02-template-declaration-grammar/README.md)
- [limit/format inventory](../../../03-limit-format-raw-inventory/README.md)

## Checklist

- [ ] Bind exact authority, cache, domain map, and declaration grammar identities.
- [ ] Extract only formal declarations with physical page, section, and source-order anchors.
- [ ] Route formats, completeness, sampling behavior, pixel layout, limits, and state out of command facts.
- [ ] Reject extensions, desktop/lower profiles, registry data, guessed templates, and ESSL semantics.
- [ ] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not prove texture formats, sampling behavior, guest/browser rendering, support, conformance,
certification, or performance.
