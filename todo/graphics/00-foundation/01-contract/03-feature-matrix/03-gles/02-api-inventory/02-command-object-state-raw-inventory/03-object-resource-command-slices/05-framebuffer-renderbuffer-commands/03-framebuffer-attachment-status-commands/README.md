# F03.3.2.2.3.5.3 — Framebuffer attachment and status commands

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F03.3.2.2.3.5.3
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: [receipt](evidence.md)

## Outcome

A bounded raw inventory covers only assigned framebuffer attachment and status declarations on p.256,
pp.258–260, and p.269. Attachment effects, image selection, completeness, formats, and returned status stay out.

## Starting points

- [framebuffer/renderbuffer parent](../README.md)
- [declaration grammar](../../../02-template-declaration-grammar/README.md)
- [limit/format inventory](../../../../03-limit-format-raw-inventory/README.md)

## Checklist

- [x] Bind exact authority, cache, framebuffer/renderbuffer domain, grammar, and unavailable-ledger identities.
- [x] Extract only `FramebufferRenderbuffer`, the three `FramebufferTexture*` literals, and `CheckFramebufferStatus`.
- [x] Preserve discontinuous source windows, the p.256 boundary after §9.2.7, section headings, source order, and the non-`void` status return form.
- [x] Route attachment effects, image/level/layer selection, completeness, formats, return values, and ESSL semantics out.
- [x] Reject object/storage/query families, extensions, registry data, guessed templates, and ESSL imports.
- [x] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not establish attachment behavior, completeness, status values, format behavior,
guest/browser rendering, support, conformance, certification, or performance.
