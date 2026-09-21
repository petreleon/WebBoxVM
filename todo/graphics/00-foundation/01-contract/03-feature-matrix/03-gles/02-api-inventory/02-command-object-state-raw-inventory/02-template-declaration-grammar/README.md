# F03.3.2.2.2 — Normalize literal and templated declarations

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.3.2.2.2
Depends: F03.3.2.1, F03.3.2.2.1
Evidence: pending

## Outcome

One bounded grammar records how the GLES 3.2 PDF's formal command declarations become literal raw names,
without importing a registry, guessing overloads, or interpreting shader-language semantics.

## Starting points

- [domain classification](../01-command-domain-classification/README.md)
- [verified normative-PDF cache](../../01-normative-pdf-cache/README.md)
- [source-authority boundary](../../../01-source-authority/README.md)

## Checklist

- [ ] Bind the exact source identity, locator grammar, and `command-state` authority decision.
- [ ] Anchor literal notation and every sealed `void NAME{...}` declaration form in the PDF before any command slice consumes it.
- [ ] Define deterministic expansion, C-binding prefix, source-order, and duplicate-rejection rules.
- [ ] Reject guessed variants, registry signatures, extensions, desktop GL, lower versions, and ESSL semantics.
- [ ] Keep grammar metadata distinct from a command fact, state transition, limit, format, or Matrix row.
- [ ] Self-hash the grammar and attach a no-claim receipt with focused hostile checks.

## Verification

The grammar is an extraction rule only. It does not establish an implemented command, state behavior,
guest/browser result, conformance, certification, or performance.
