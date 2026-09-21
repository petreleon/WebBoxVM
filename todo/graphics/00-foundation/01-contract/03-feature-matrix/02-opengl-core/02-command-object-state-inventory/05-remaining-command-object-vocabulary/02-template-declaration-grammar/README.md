# F03.2.2.5.2 — Normalize literal and templated command declarations

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.2.5.2
Depends: F03.2.1, F03.2.2.1, F03.2.2.5.1
Evidence: [evidence](evidence.md)

## Outcome

A fail-closed grammar normalizes only literal and formally templated OpenGL 4.6 core declarations before
family extraction; it does not itself count as coverage of any command family.

## Starting points

- [command-domain classifier](../01-command-domain-classification/README.md)
- [normative PDF cache](../../01-normative-pdf-cache/README.md)
- [direct-creation baseline](../../02-command-object-raw-inventory/README.md)

## Checklist

- [x] Bind the exact cache; distinguish the §2.1 C-prefix witness on page 32 from §2.2 notation on pages 32–34, and anchor §7.6.1 matrix templates on page 163.
- [x] Apply the documented C-binding `gl` prefix only where the PDF's rule permits it.
- [x] Encode the explicit template-expansion grammar, including `Uniform{1234}{if}`, with source anchors.
- [x] Reject guessed suffixes, unexpanded templates, duplicate generated names, and ambiguous declarations.
- [x] Reject registry, headers, lower profiles, extensions, GLSL, and compatibility material as substitute grammar.
- [x] Emit only a self-hashed normalization rule set with zero Matrix/CTS/claim fields and a receipt.

## Verification

This grammar validates later raw declaration extraction. It does not create a command inventory, infer
semantics, or make a support, conformance, guest/browser, certification, or performance claim.
