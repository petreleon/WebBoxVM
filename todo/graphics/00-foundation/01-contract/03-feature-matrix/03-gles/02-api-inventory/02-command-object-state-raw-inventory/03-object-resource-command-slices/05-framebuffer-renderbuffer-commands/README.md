# F03.3.2.2.3.5 — Extract framebuffer and renderbuffer commands

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F03.3.2.2.3.5
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: pending

## Outcome

A bounded raw inventory records formal framebuffer and renderbuffer command declarations; attachment,
completeness, conversion, and format behavior remain outside this declaration-only source slice.

## Starting points

- [object/resource slice list](../README.md)
- [declaration grammar](../../02-template-declaration-grammar/README.md)
- [limit/format inventory](../../../03-limit-format-raw-inventory/README.md)

## Checklist

- [ ] Bind exact authority, cache, domain map, and declaration grammar identities.
- [ ] Extract only formal declarations with physical page, section, and source-order anchors.
- [ ] Route attachment, completeness, conversions, formats, limits, and lifecycle out of command facts.
- [ ] Reject extensions, desktop/lower profiles, registry data, guessed templates, and ESSL semantics.
- [ ] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not prove framebuffer completeness, format behavior, guest/browser rendering, support,
conformance, certification, or performance.
