# F02.5.4 — Cut over inventory and source consumers atomically

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.5.4
Depends: F02.5.2, F02.5.3
Evidence: pending

## Outcome

The inventory, lock, profile-source gate, matrix contracts, and F05 registration consume the same
role-aware source records. A stale, mixed, or scope-misrepresented record cannot make F03 or F05 pass.

## Starting points

- [F02 inventory](../../01-input-inventory/manifest.toml)
- [F03 profile gate](../../../03-feature-matrix/01-profile-scope/validate_profile_scope.py)
- [F05 generic runner](../../../../02-reproducibility/02-check-runner/01-generic-runner/README.md)

## Checklist

- [x] [F02.5.4.1 — Seal the role-aware source contract](01-role-aware-source-contract/README.md)
- [ ] [F02.5.4.2 — Bind the F03 profile gate and matrix](02-f03-gate-and-matrix-binding/README.md)
- [ ] [F02.5.4.3 — Expose the future F05 adapter and aggregate receipt](03-future-f05-adapter-and-receipt/README.md)

## Verification

- F03 may advance from `inventory-sources-incomplete` to `matrix-incomplete` only after all roles are
  admitted. No source cutover marks a guest API, browser path, CTS run, certification, or performance pass.

## Split rationale

The successor inventory/lock, F03's schema-breaking consumer migration, and the later F05-facing
adapter have independent failure boundaries. Separating them prevents the historical 17-input F02.1
inventory and its generic 8 MiB transport rule from being mistaken for a full-suite admission contract.
