# F02.5.3.2.1 — Ledger the GL flat sequence

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.5.3.2.1
Depends: F02.5.3.1
Evidence: pending

## Outcome

The immutable `gl46-main.txt` root is parsed as its actual ordered flat case sequence,
with a count and sequence digest. It remains a Khronos full-suite root; the local ledger
is a no-claim WebBoxVM observation.

## Starting points

- [canonical root](../../01-canonical-full-suite-roots/full_suite_roots.py)
- [reviewed GL candidate](../../../../04-profile-source-inputs/candidate_catalog.py)

## Checklist

- [ ] Fetch the exact root into a fresh external cache and verify its pinned identity.
- [ ] Parse 19,714 unique ordered nonblank case lines without normalizing their bytes.
- [ ] Bind count and sequence digest to the root identity.
- [ ] Reject omitted, duplicated, reordered, substituted, or malformed cases.
- [ ] Keep the raw root and local observation under distinct authorities.

## Verification

`make graphics-gl-flat-ledger-test` must exercise positive, boundary, and hostile
sequence cases. The receipt must state zero CTS executions and no support or conformance
claim.
