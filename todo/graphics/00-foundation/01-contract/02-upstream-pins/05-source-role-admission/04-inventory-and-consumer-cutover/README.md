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

- [ ] Admit all six required source roles atomically and renew the inventory lock and cache contract.
- [ ] Bind each F03 requirement to its normative root and full-suite record with exact revision and digest.
- [ ] Register local transforms and engineering maps as non-conformance auxiliary evidence only.
- [ ] Require any missing mandatory role, stale pin, or unmatched full-suite root to keep the profile blocked.
- [ ] Add positive and hostile cutover tests, then run the fresh-cache and downstream consumer checks.
- [ ] Attach an aggregate receipt and update immediate parent checkboxes only after all checks pass.

## Verification

- F03 may advance from `inventory-sources-incomplete` to `matrix-incomplete` only after all roles are
  admitted. No source cutover marks a guest API, browser path, CTS run, certification, or performance pass.
