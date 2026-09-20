# F02.5.4.2 — Bind the F03 profile gate and matrix

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.4.2
Depends: F02.5.4.1
Evidence: pending

## Outcome

F03 consumes only the sealed role-aware contract and exact ordered normative-root/full-suite-root
pairs. Legacy manifest aliases cannot silently resolve, and a complete source contract changes only
the blocker from `inventory-sources-incomplete` to `matrix-incomplete`.

## Starting points

- [F03 profile gate](../../../../03-feature-matrix/01-profile-scope/validate_profile_scope.py)
- [F03 source requirements](../../../../03-feature-matrix/01-profile-scope/source_requirements.json)
- [F03 matrix contract](../../../../03-feature-matrix/01-profile-scope/matrix_contract.py)

## Checklist

- [ ] Migrate F03 source requirements and profile scope to a fail-closed role-aware schema.
- [ ] Bind OpenGL, GLES, and Vulkan to their exact normative root and canonical full-suite root.
- [ ] Reject legacy aliases, missing roles, duplicate or reordered pairs, and cross-profile bindings.
- [ ] Reject wrong kind, stale revision/digest, filtered selector, root-only suite, or auxiliary substitute.
- [ ] Make matrix validation consume the same binding API rather than parallel source constants.
- [ ] Add positive and hostile tests proving the only eligible next blocker is `matrix-incomplete`.

## Verification

All profiles remain blocked. A valid source contract is prerequisite evidence only: it cannot claim a
guest API, browser path, CTS execution, certification, supported profile, or performance result.
