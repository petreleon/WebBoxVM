# F03.3.2.2.3.4.2 — Texture image, copy, and subimage commands

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F03.3.2.2.3.4.2
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: pending

## Outcome

A bounded raw inventory covers only the assigned texture-image, copy-image, and subimage declarations in
sections 8.5–8.6. Formats, pixels, framebuffer behavior, dimensions, and image contents stay out.

## Starting points

- [texture/sampler parent](../README.md)
- [declaration grammar](../../../02-template-declaration-grammar/README.md)
- [limit/format inventory](../../../../03-limit-format-raw-inventory/README.md)

## Checklist

- [ ] Bind exact authority, cache, texture domain, grammar, and unavailable-ledger identities.
- [ ] Extract only assigned literals from pp.175, 184, 185–187, and 191–192 with source-order anchors.
- [ ] Preserve the p.185–187 and p.191–192 declaration spans without promoting formats, pixels, or behavior.
- [ ] Reject compressed/storage/parameter families, extensions, registry data, guessed declarations, and ESSL imports.
- [ ] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not establish image allocation, copies, pixel layout, framebuffer behavior, guest/browser
rendering, support, conformance, certification, or performance.
