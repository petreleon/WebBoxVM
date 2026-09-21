# F03.3.2.2.3.4.4 — Texture parameter and query templates

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F03.3.2.2.3.4.4
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: [receipt](evidence.md)

## Outcome

A bounded raw inventory expands only the assigned texture parameter and query templates in sections
8.10–8.11.3. Parameter values, query results, types, texture state, and level semantics stay out.

## Starting points

- [texture/sampler parent](../README.md)
- [declaration grammar](../../../02-template-declaration-grammar/README.md)
- [unavailable source ledger](../../../../04-unavailable-language-extension-ledger/README.md)

## Checklist

- [x] Bind exact authority, cache, texture domain, grammar, and unavailable-ledger identities.
- [x] Expand only the assigned p.206, p.209, and p.210 formal templates with source-order anchors.
- [x] Route parameter values, query results, types, texture state, levels, and ESSL semantics out of facts.
- [x] Reject sampler templates/queries, extensions, registry data, guessed templates, and ESSL-source imports.
- [x] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declaration forms do not establish texture parameters, query results, state behavior, guest/browser rendering,
support, conformance, certification, or performance.
