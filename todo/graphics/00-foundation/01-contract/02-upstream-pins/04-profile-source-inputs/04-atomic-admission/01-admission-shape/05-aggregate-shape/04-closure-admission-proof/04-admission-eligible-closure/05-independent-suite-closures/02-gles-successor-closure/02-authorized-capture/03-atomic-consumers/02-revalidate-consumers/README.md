# F02.4.4.1.5.4.4.5.2.2.3.2 — Revalidate the F02 and F03 consumers

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.2.2.3.2
Depends: F02.4.4.1.5.4.4.5.2.2.3.1
Evidence: pending

Prerequisite lists: the [sealed capture binding](../01-bind-sealed-capture/README.md), the
[blocked-state receipt](../../../../../../03-blocked-state-receipt/README.md), and the
[F03 source requirements](../../../../../../../../../../../../03-feature-matrix/01-profile-scope/source_requirements.json).

## Outcome

Revalidate the unchanged F02 active inventory, proof receipt, and F03 source gate against the same
GLES capture binding. The result must prove that the active v2 identity and F03's six absent required
inputs remain unchanged until an authorized aggregate transition renews every mandatory consumer.

## Starting points

- [active inventory](../../../../../../../../../../../01-input-inventory/manifest.toml)
- [blocked-state receipt](../../../../../../03-blocked-state-receipt/blocked_state_receipt.py)
- [F03 gate](../../../../../../../../../../../../03-feature-matrix/01-profile-scope/validate_profile_scope.py)

## Checklist

- [ ] Revalidate exact active-v2 inventory identity and the non-admitting six-role proof receipt.
- [ ] Revalidate F03's ordered source requirements and its expected `inventory-sources-incomplete` result.
- [ ] Keep the captured GLES closure outside active F02 and F03 until a whole-transition policy authorizes it.
- [ ] Reject active aliasing, F03 omission/substitution, stale locks, and a claimed partial consumer update.
- [ ] Add focused positive and hostile tests while preserving all feature/profile states as blocked.

## Verification

- An unchanged consumer is intentional evidence here, not an incomplete test run.
- This child cannot change an inventory, F03 status, API feature row, or implementation claim.
