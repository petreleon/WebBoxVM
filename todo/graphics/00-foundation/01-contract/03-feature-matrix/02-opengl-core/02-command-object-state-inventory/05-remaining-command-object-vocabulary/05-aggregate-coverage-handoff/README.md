# F03.2.2.5.5 — Aggregate command-vocabulary coverage and handoff guard

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F03.2.2.5.5
Depends: F03.2.2.2, F03.2.2.5.1, F03.2.2.5.2, F03.2.2.5.3, F03.2.2.5.4
Evidence: pending

## Outcome

One aggregate receipt proves that every classified core command-declaration family is covered or visibly
blocked, preserves the original direct-creation artifact, and rejects raw facts from Matrix import.

## Starting points

- [direct-creation baseline](../../02-command-object-raw-inventory/README.md)
- [command classifier](../01-command-domain-classification/README.md)
- [raw handoff guard](../../04-raw-handoff-and-import-guard/README.md)

## Checklist

- [ ] Bind all child artifact identities and the F03.2.2.2 direct-creation baseline without replacement.
- [ ] Require every classified command family to be covered by a child or explicit blocker exactly once.
- [ ] Reject duplicate `Create*` facts, unclassified families, missing artifacts, and reordered coverage maps.
- [ ] Conclude at most `command_declaration_domain_complete`; retain `state_lifecycle_domain_complete=false`.
- [ ] Reject owner, test, Matrix, CTS, support, guest/browser, certification, and performance promotion.
- [ ] Attach a no-claim receipt for F03.2.2.4.

## Verification

This is a raw-source coverage gate, not evidence of an implemented OpenGL command path or the broader
command/object/state domain required by F03.2.2.
