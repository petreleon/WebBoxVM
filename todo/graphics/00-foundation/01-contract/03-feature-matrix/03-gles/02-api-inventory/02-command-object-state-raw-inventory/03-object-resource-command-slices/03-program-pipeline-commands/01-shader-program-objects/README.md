# F03.3.2.2.3.3.1 — Shader and program objects

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F03.3.2.2.3.3.1
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: pending

## Outcome

A bounded raw inventory records only the formal shader-object and program-object declarations assigned to
sections 7.1–7.3. Shader text, compilation, linking, binary behavior, and object lifetime semantics stay out.

## Starting points

- [program/pipeline split](../README.md)
- [declaration grammar](../../../02-template-declaration-grammar/README.md)
- [unavailable source ledger](../../../../04-unavailable-language-extension-ledger/README.md)

## Checklist

- [ ] Bind exact authority, cache, object-declaration domain, grammar, and unavailable-ledger identities.
- [ ] Extract assigned formal declarations with physical page, section, and source-order anchors only.
- [ ] Route shader text, compilation, linking, binaries, lifecycle, and ESSL semantics out of command facts.
- [ ] Reject extensions, desktop/lower profiles, registry data, guessed templates, and ESSL-source imports.
- [ ] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not establish shader or program behavior, guest/browser execution, support, conformance,
certification, or performance.
