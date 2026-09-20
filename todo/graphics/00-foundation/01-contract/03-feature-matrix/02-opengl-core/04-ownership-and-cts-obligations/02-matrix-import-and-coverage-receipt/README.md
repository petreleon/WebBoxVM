# F03.2.4.2 — Import the matrix and receipt coverage

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F03.2.4.2
Depends: F03.2.4.1
Evidence: pending

## Outcome

Only complete source facts with assigned ownership and independent reference obligations are converted to
the F03 Matrix v2 schema. The resulting coverage receipt remains blocked until later implementation and
execution evidence exists.

## Starting points

- [ownership and reference obligations](../01-ownership-and-reference-obligations/README.md)
- [F03 v2 matrix contract](../../../01-profile-scope/matrix_contract_v2.py)
- [F05.2 registration boundary](../../../../../02-reproducibility/02-check-runner/02-profile-bound-registration/README.md)

## Checklist

- [ ] Convert only the checked raw fact families plus their explicit owner and reference-obligation maps.
- [ ] Preserve the exact normative and full-suite role identities, source locators, conditions, and
  blocked status on every Matrix v2 row.
- [ ] Reject an absent family, owner, obligation, evidence receipt, source mismatch, duplicate row, or
  falsely supported status.
- [ ] Keep unassigned facts visible outside the imported matrix with their specific blocker; do not drop
  them to make coverage look complete.
- [ ] Aggregate source counts, owners, selectors, blockers, and zero CTS executions in a no-claim receipt.
- [ ] Run focused import and hostile checks before handing the coverage map to F03.2.5.

## Verification

Matrix import proves only that planned facts are consistently attributed. F05.2 registration and later
execution remain separate; this child makes no API-support, guest, browser, conformance, certification,
or performance claim.
