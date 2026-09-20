# F03.2.5 — Register checks and aggregate the receipt

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F03.2.5
Depends: F03.2.2, F03.2.3, F03.2.4, F05.2
Evidence: pending

## Outcome

The complete OpenGL 4.6 inventory, coverage obligations, and profile-bound F05 registrations are joined in
one reviewable receipt. Missing device, browser, guest, suite, or source prerequisites remain blocked.

## Starting points

- [command/object/state inventory](../02-command-object-state-inventory/README.md)
- [limit/format/shader inventory](../03-limit-format-shader-inventory/README.md)
- [ownership and CTS obligations](../04-ownership-and-cts-obligations/README.md)
- [F05.2 profile-bound registration](../../../../02-reproducibility/02-check-runner/02-profile-bound-registration/README.md)

## Checklist

- [ ] Register every admitted row family through F05.2 with its exact OpenGL profile, source identity,
  expected nonzero count, command, artifact location, and mandatory prerequisites.
- [ ] Reject a stale source, wrong profile, empty command, missing row family, missing owner, missing
  independent test obligation, or unavailable prerequisite presented as PASS.
- [ ] Run the focused source-boundary, extraction, coverage, and F05 registration checks; preserve actual
  blocked results and commands in the receipt.
- [ ] Aggregate source identities, row counts, extension decisions, owners, test obligations, and F05
  registrations into one self-consistent review record.
- [ ] Mark the F03.2 parent complete only after reviewing every child receipt and every blocked mandatory
  row; otherwise retain `matrix-incomplete`.
- [ ] Attach the aggregate no-claim receipt with exact commands, outputs, and zero implied CTS executions.

## Verification

Registration proves only that planned checks are bound to frozen inputs. It cannot turn an unavailable
browser, native device, guest image, or conformance suite into a pass, nor claim OpenGL support,
certification, or near-native performance.
