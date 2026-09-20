# F02.5.4.2 — Bind the F03 profile gate and matrix

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.4.2
Depends: F02.5.4.1
Evidence: [receipt](evidence.md)

## Outcome

F03 consumes only the sealed role-aware contract and exact ordered normative-root/full-suite-root
pairs. Legacy manifest aliases cannot silently resolve, and a complete source contract changes only
the blocker from `inventory-sources-incomplete` to `matrix-incomplete`.

## Starting points

- [F03 profile gate](../../../../03-feature-matrix/01-profile-scope/validate_profile_scope.py)
- [F03 v2 source requirements](../../../../03-feature-matrix/01-profile-scope/source_requirements_v2.json)
- [F03 v2 matrix contract](../../../../03-feature-matrix/01-profile-scope/matrix_contract_v2.py)
- [sealed source lock](../01-role-aware-source-contract/source_contract.lock)

## Checklist

- [x] Migrate F03 source requirements and profile scope to a fail-closed role-aware schema.
- [x] Bind OpenGL, GLES, and Vulkan to their exact normative root and canonical full-suite root.
- [x] Reject legacy aliases, missing roles, duplicate or reordered pairs, and cross-profile bindings.
- [x] Reject wrong kind, stale revision/digest, filtered selector, root-only suite, or auxiliary substitute.
- [x] Make matrix validation consume the same binding API rather than parallel source constants.
- [x] Add positive and hostile tests proving the only eligible next blocker is `matrix-incomplete`.

## Verification

All profiles remain blocked. A valid source contract is prerequisite evidence only: it cannot claim a
guest API, browser path, CTS execution, certification, supported profile, or performance result.

The v1 files remain an explicitly historical F02.4 snapshot. The active runner selects only v2 and
never falls back to the legacy manifest or its aliases.
