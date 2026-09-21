# F03.3.2.2.3.3.1 — Shader, program, pipeline, and binary literals

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F03.3.2.2.3.3.1
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: [receipt](evidence.md)

## Outcome

A bounded raw inventory records assigned literal shader/program object, program-interface, pipeline, and binary
declarations from sections 7.1–7.5. Compilation, linking, matching, validation, binary, and lifecycle behavior stay out.

## Starting points

- [program/pipeline split](../README.md)
- [declaration grammar](../../../02-template-declaration-grammar/README.md)
- [unavailable source ledger](../../../../04-unavailable-language-extension-ledger/README.md)

## Checklist

- [x] Bind exact authority, cache, object-declaration domain, grammar, and unavailable-ledger identities.
- [x] Extract assigned literal declarations with physical page, section, and source-order anchors only.
- [x] Route compilation, linking, matching, validation, binaries, and lifecycle behavior out of command facts.
- [x] Reject extensions, desktop/lower profiles, registry data, guessed templates, and ESSL-source imports.
- [x] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not establish shader, program, pipeline, or binary behavior, guest/browser execution, support,
conformance, certification, or performance.
