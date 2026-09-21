# F03.2.2.5.3.2 — Buffer command declarations

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F03.2.2.5.3.2
Depends: F03.2.1, F03.2.2.1, F03.2.2.2, F03.2.2.5.2
Evidence: pending

## Outcome

Raw buffer command declarations from §§6.1–6.7 (physical pages 81–105) are captured separately from the
three prior buffer state facts and without inferring buffer lifecycle semantics.

## Starting points

- [object/resource slice list](../README.md)
- [buffer state raw slice](../../../03-state-and-lifecycle-raw-inventory/01-buffer-binding-lifecycle-raw-slice/README.md)
- [normative PDF cache](../../../01-normative-pdf-cache/README.md)

## Checklist

- [ ] Bind the exact cache, classifier, grammar, and existing buffer state artifact.
- [ ] Extract only assigned formal command declarations with physical pages, sections, and source order.
- [ ] Route binding/deletion transitions and numeric properties to their existing raw state or limit inventories.
- [ ] Reject duplicate creation facts, unanchored templates, extensions, compatibility, registry, and profile substitutions.
- [ ] Self-hash the bounded artifact; keep Matrix/CTS/owner/claim fields absent or zero and attach a receipt.

## Verification

The result is command-signature source data only; it does not prove buffer correctness, lifetime behavior,
guest/browser behavior, conformance, certification, or performance.
