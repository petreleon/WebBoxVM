# F03.2.2.5.4.1 — Global execution and synchronization commands

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F03.2.2.5.4.1
Depends: F03.2.1, F03.2.2.1, F03.2.2.5.1, F03.2.2.5.2
Evidence: pending

## Outcome

Global execution and synchronization command declarations from §§2.3.1–2.3.3 (pages 38–43) and §7.13
(pages 183 and 187) are recorded without inferring ordering, memory visibility, or execution semantics.

## Starting points

- [non-object slice list](../README.md)
- [template grammar](../../02-template-declaration-grammar/README.md)
- [normative PDF cache](../../../01-normative-pdf-cache/README.md)

## Checklist

- [ ] Bind the exact cache, classifier, and declaration grammar.
- [ ] Extract only assigned formal declarations with page, section, and source-order anchors.
- [ ] Keep ordering, visibility, state, and synchronization behavior out of this declaration inventory.
- [ ] Reject registry, extension, compatibility, lower-profile, guessed-template, and unanchored sources.
- [ ] Self-hash the raw artifact; retain zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Declarations alone do not prove synchronization correctness, guest/browser behavior, support, conformance,
certification, or performance.
