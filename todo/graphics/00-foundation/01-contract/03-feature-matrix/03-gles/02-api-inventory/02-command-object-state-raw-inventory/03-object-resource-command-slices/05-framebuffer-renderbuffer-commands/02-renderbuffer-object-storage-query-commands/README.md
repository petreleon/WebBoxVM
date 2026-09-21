# F03.3.2.2.3.5.2 — Renderbuffer lifecycle, storage, and queries

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F03.3.2.2.3.5.2
Depends: F03.3.2.2.1, F03.3.2.2.2
Evidence: [evidence](evidence.md)

## Outcome

A bounded raw inventory covers only assigned renderbuffer lifecycle, storage, and query declarations on pp.252–256.
Storage allocation behavior, multisampling behavior, formats, dimensions, contents, values, and attachment behavior stay out.

## Starting points

- [framebuffer/renderbuffer parent](../README.md)
- [declaration grammar](../../../02-template-declaration-grammar/README.md)
- [limit/format inventory](../../../../03-limit-format-raw-inventory/README.md)

## Checklist

- [x] Bind exact authority, cache, framebuffer/renderbuffer domain, grammar, and unavailable-ledger identities.
- [x] Extract only assigned renderbuffer bind/object/storage/query literals from pp.252–256.
- [x] Preserve source order, section boundaries, the p.256 boundary before §9.2.7, and the non-`void` `IsRenderbuffer` return form.
- [x] Route allocation, multisampling, formats, dimensions, contents, query values, attachments, and ESSL semantics out.
- [x] Reject framebuffer/attachment/status families, extensions, registry data, guessed declarations, and ESSL imports.
- [x] Self-hash raw facts with zero Matrix/CTS/owner/claim fields and attach a receipt.

## Verification

Raw declarations do not establish renderbuffer storage, multisampling, format behavior, query results,
guest/browser rendering, support, conformance, certification, or performance.
