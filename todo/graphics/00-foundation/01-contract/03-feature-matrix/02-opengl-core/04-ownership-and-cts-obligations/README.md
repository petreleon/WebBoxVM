# F03.2.4 — Map implementation ownership and CTS obligations

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.2.4
Depends: F03.2.2, F03.2.3
Evidence: pending

## Outcome

Every admitted OpenGL 4.6 matrix row has a downstream implementation owner and an independent native,
guest, or conformance reference-test obligation. Missing mappings stay visible rather than shrinking the
profile.

## Starting points

- [command/object/state inventory](../02-command-object-state-inventory/README.md)
- [limit/format/shader inventory](../03-limit-format-shader-inventory/README.md)
- [admitted OpenGL CTS root](../../../02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/README.md)

## Checklist

- [ ] Merge the two inventory families and reject a missing, duplicate, or source-incompatible mandatory
  row before assigning ownership.
- [ ] Link every mandatory row to a concrete downstream implementation task or a visibly unresolved
  owner; do not replace a missing owner with a generic transport milestone.
- [ ] Give each row an independent native, guest, or canonical conformance-test obligation with an exact
  selector or documented reason it remains blocked.
- [ ] Classify bounded VirGL evidence only where its source supports the exact row; retain unsupported or
  unknown behavior as blocked and never substitute a lower bring-up API level.
- [ ] Reject an empty owner, generic test plan, stale suite root, wrong profile, or row marked supported
  without independent evidence.
- [ ] Attach a coverage receipt with zero implied CTS executions and no qualification claims.

## Verification

The coverage map plans implementation and testing; it does not execute CTS or establish guest, browser,
certification, support, or performance behavior.
