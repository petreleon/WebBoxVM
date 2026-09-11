# F02.5.3.2 — Ledger the complete GL and GLES suites

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.3.2
Depends: F02.5.3.1
Evidence: [aggregate receipt](03-gl-gles-no-claim-receipt/evidence.md)

## Outcome

The ledger preserves the real shapes of the admitted OpenGL and GLES suites rather
than inventing uniform members. GL is an ordered flat case list; GLES includes every
root-referenced list, configuration, and gles32-khr-glesext boundary asset.

## Starting points

- [GL/GLES candidate records](../../../04-profile-source-inputs/candidate_catalog.py)
- [GLES closure audit](../../../04-profile-source-inputs/02-gles-input-audit/cts_closure.json)

## Checklist

- [x] [F02.5.3.2.1 — Ledger the GL flat sequence](01-gl-flat-ledger/README.md)
- [x] [F02.5.3.2.2 — Ledger the GLES complete closure](02-gles-full-closure-ledger/README.md)
- [x] [F02.5.3.2.3 — Receipt the GL/GLES ledgers](03-gl-gles-no-claim-receipt/README.md)

## Verification

The ledger replays every root reference exactly. It does not impose the local 8 MiB
transform cap on an upstream GL/GLES source or turn the ledger into a test result.

## Split rationale

GL is one ordered flat case file, while GLES has a generated XML descriptor, repeated
configurations, and a boundary list whose semantics are deliberately not inferred. A
separate receipt can therefore prove both full shapes without treating the old core map
as the complete upstream root.
