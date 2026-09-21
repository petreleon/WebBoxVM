# F03.3.2.2.3.4.3 — Compressed, storage, buffer, mipmap, and image commands

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F03.3.2.2.3.4.3
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: [receipt](evidence.md)

## Outcome

A bounded raw inventory covers only assigned compressed-image, storage, texture-buffer, mipmap, and image-binding
declarations across Chapter 8. Formats, storage, synchronization, image access, and execution semantics stay out.

## Starting points

- [texture/sampler parent](../README.md)
- [declaration grammar](../../../02-template-declaration-grammar/README.md)
- [limit/format inventory](../../../../03-limit-format-raw-inventory/README.md)

## Checklist

- [x] Bind exact authority, cache, texture domain, grammar, and unavailable-ledger identities.
- [x] Extract only assigned literals from pp.195, 199, 201, 203–204, 222, 226–227, and 233.
- [x] Preserve formal source order while routing formats, storage, barriers, image access, and behavior out.
- [x] Reject image/copy/parameter families, extensions, registry data, guessed declarations, and ESSL imports.
- [x] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not establish compression, storage allocation, mipmap generation, image access, synchronization,
guest/browser rendering, support, conformance, certification, or performance.
