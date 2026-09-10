# F02.4.4.1.5.2.3.5.2.1 — Freeze the proof-only grammar and binder

[Parent task](../README.md)

Task: F02.4.4.1.5.2.3.5.2.1
Depends: F02.4.4.1.5.2.3.3.2, F02.4.4.1.5.2.3.3.3
Evidence: [receipt](evidence.md)

## Outcome

A bounded, proof-only event grammar and binder accepts an observed lineage relation only when each generated input
has one closed writer, a safe finalization path, and conservative process dependencies from the sealed raw scope.

## Starting points

- [event parser](../lineage_events.py)
- [lineage binder](../lineage_bind.py)
- [sealed capture bridge](../lineage_capture.py)

## Checklist

- [x] Define bounded process, exec, exit, read, write, rename, close, and final records with exact schemas.
- [x] Bind every final generated member to one closed writer and a causal, ordered producer-input set.
- [x] Reject path escape, ambiguity, mutation, cycles, stale scope, and invalid process lifecycle transitions.
- [x] Keep the implementation proof-only and rebind receipts to a capture path rather than caller-supplied scope.

## Verification

- Focused positive and hostile tests exercise the grammar, binder, capture anchor, and receipt self-hash.
- This grammar accepts no Docs build result by itself and does not establish source or output provenance.
