# F02.5.3.2.3 — Receipt the GL/GLES ledgers

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.5.3.2.3
Depends: F02.5.3.2.1, F02.5.3.2.2
Evidence: pending

## Outcome

One local receipt binds the independently replayed GL and GLES observations to their
immutable Khronos roots, while denying CTS execution and all qualification claims.

## Starting points

- [GL ledger](../01-gl-flat-ledger/README.md)
- [GLES ledger](../02-gles-full-closure-ledger/README.md)
- [source-role policy](../../../01-authority-and-transform-boundary/source-role-policy.md)

## Checklist

- [ ] Require the exact GL and GLES ledger identities and source revisions.
- [ ] Report GL cases, GLES lists, configurations, and the glesext boundary separately.
- [ ] Reject a forged, incomplete, reordered, or positive-claim receipt.
- [ ] Preserve the root-versus-local-authority boundary.

## Verification

`make graphics-gl-gles-ledger-receipt-test` must reject mutations and show a receipt
with `cts_executions: 0` and every qualification claim false.
