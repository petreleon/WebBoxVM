# F05 — Create a reproducible graphics verification runner

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: F05
Depends: F01, F02
Evidence: pending

Prerequisite lists: [F01](../../01-contract/01-baseline/README.md), [F02](../../01-contract/02-upstream-pins/README.md).

## Outcome

Each future leaf can name an exact one-command check and produce an inspectable receipt once its
profile-bound registration is admitted.

## Starting points

- [Makefile](../../../../../Makefile)
- [scripts/serve_web.py](../../../../../scripts/serve_web.py)
- [scripts/virgl_guest_transport_smoke.sh](../../../../../scripts/virgl_guest_transport_smoke.sh)
- [emulator/tests/source_file_limits.rs](../../../../../emulator/tests/source_file_limits.rs)

## Checklist

- [ ] [F05.1 — Implement the profile-independent runner substrate](01-generic-runner/README.md)
- [ ] [F05.2 — Register profile-bound graphics checks](02-profile-bound-registration/README.md)

## Verification

- F05 remains incomplete until both children pass and their combined verification preserves failing-child,
  empty-selection and missing-prerequisite behavior.
- Existing make test and make web-pkg remain runnable lanes; the native guest smoke remains transport-only.

Register the exact task check through F05.2 before profile-bound implementation completion. Bootstrap
F01/F02/F05.1 use direct reproducible commands and receipts until the generic substrate exists. Record
commands, nonzero test counts (or explicit design checks), output and expected results; a planned check
is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
