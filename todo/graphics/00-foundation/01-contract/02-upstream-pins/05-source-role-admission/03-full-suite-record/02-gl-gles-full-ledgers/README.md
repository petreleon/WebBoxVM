# F02.5.3.2 — Ledger the complete GL and GLES suites

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.3.2
Depends: F02.5.3.1
Evidence: pending

## Outcome

The ledger preserves the real shapes of the admitted OpenGL and GLES suites rather
than inventing uniform members. GL is an ordered flat case list; GLES includes every
root-referenced list, configuration, and gles32-khr-glesext boundary asset.

## Starting points

- [GL/GLES candidate records](../../../04-profile-source-inputs/candidate_catalog.py)
- [GLES closure audit](../../../04-profile-source-inputs/02-gles-input-audit/cts_closure.json)

## Checklist

- [ ] Record the ordered GL flat-case sequence with count and sequence digest.
- [ ] Record all GLES referenced lists, configurations, revisions, bytes, and digests.
- [ ] Include gles32-khr-glesext.txt in the full-root ledger without claiming its semantics are mandatory.
- [ ] Reject an omitted, duplicate, reordered, substituted, or misclassified item.
- [ ] Keep raw upstream content and local engineering maps under distinct roles.

## Verification

The ledger replays every root reference exactly. It does not impose the local 8 MiB
transform cap on an upstream GL/GLES source or turn the ledger into a test result.
