# F03.3.2.2.3.4.1 — Texture and sampler objects plus parameter templates

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F03.3.2.2.3.4.1
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: pending

## Outcome

A bounded raw inventory covers only texture/sampler object declarations and the assigned sampler-parameter
templates from sections 8.1–8.2. Parameter values, object state, sampling, and lifetime semantics stay out.

## Starting points

- [texture/sampler parent](../README.md)
- [declaration grammar](../../../02-template-declaration-grammar/README.md)
- [unavailable source ledger](../../../../04-unavailable-language-extension-ledger/README.md)

## Checklist

- [ ] Bind exact authority, cache, texture/sampler domains, grammar, and unavailable-ledger identities.
- [ ] Extract p.157–161 object literals and only the three p.160 sampler-parameter templates.
- [ ] Preserve formal source locations and expansion order while routing values, state, sampling, and lifetime out.
- [ ] Reject sampler queries, texture-unit state, extensions, registry data, guessed templates, and ESSL imports.
- [ ] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not establish object validity, parameter effects, sampling behavior, guest/browser rendering,
support, conformance, certification, or performance.
