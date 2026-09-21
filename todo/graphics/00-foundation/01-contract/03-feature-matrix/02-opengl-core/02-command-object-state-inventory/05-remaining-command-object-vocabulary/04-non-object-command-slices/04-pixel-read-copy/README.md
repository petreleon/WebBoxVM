# F03.2.2.5.4.4 — Pixel read and copy commands

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F03.2.2.5.4.4
Depends: F03.2.1, F03.2.2.1, F03.2.2.5.1, F03.2.2.5.2
Evidence: pending

## Outcome

Pixel read and copy command declarations from chapter 18 (physical pages 551–567) are source-located raw
facts, while pack/unpack state, formats, errors, and transferred bytes remain outside this slice.

## Starting points

- [non-object slice list](../README.md)
- [raw limit/format route](../../../../03-limit-format-shader-inventory/README.md)
- [normative PDF cache](../../../01-normative-pdf-cache/README.md)

## Checklist

- [ ] Bind the exact cache, classifier, and declaration grammar.
- [ ] Extract only formal pixel read/copy declarations with physical-page and source-order anchors.
- [ ] Route pack/unpack state, formats, constraints, errors, and data behavior out of this inventory.
- [ ] Reject registry, extension, compatibility, lower-profile, guessed-template, and unanchored sources.
- [ ] Self-hash the raw artifact; retain zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

This is not evidence of pixel transfer correctness, guest/browser behavior, support, conformance,
certification, or performance.
