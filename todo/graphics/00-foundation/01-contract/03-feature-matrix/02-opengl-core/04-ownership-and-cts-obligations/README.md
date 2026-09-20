# F03.2.4 — Map implementation ownership and CTS obligations

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.2.4
Depends: F03.2.2, F03.2.3
Evidence: pending

## Outcome

Every admitted OpenGL 4.6 raw inventory fact gains a downstream implementation owner and an independent
native, guest, or conformance reference-test obligation before it is imported into the shared matrix.
Missing mappings stay visible rather than shrinking the profile.

## Starting points

- [command/object/state inventory](../02-command-object-state-inventory/README.md)
- [limit/format/shader inventory](../03-limit-format-shader-inventory/README.md)
- [admitted OpenGL CTS root](../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)

## Checklist

- [ ] [F03.2.4.1 — Assign ownership and reference obligations](01-ownership-and-reference-obligations/README.md)
- [ ] [F03.2.4.2 — Import the matrix and receipt coverage](02-matrix-import-and-coverage-receipt/README.md)

## Verification

The coverage map plans implementation and testing; it does not execute CTS or establish guest, browser,
certification, support, or performance behavior.

## Split rationale

F03.2.2 and F03.2.3 can establish source facts but cannot truthfully fill Matrix v2 owner and reference
fields. Ownership and independent obligations are resolved first; only then does the second child import
the verified raw families into the shared schema. F03.2.5 continues to depend on this parent rather than
on an early, partially attributed matrix.
