# F02.5.3.2.2 — Ledger the GLES complete closure

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.5.3.2.2
Depends: F02.5.3.1
Evidence: pending

## Outcome

The immutable GLES `mustpass.xml` root, every referenced list, and all 13 ordered
configurations are recorded exactly. `gles32-khr-glesext.txt` is a root-referenced
boundary member, not an inferred mandatory or optional semantic classification.

## Starting points

- [canonical root](../../01-canonical-full-suite-roots/full_suite_roots.py)
- [reviewed closure audit](../../../../04-profile-source-inputs/02-gles-input-audit/cts_closure.json)

## Checklist

- [ ] Fetch the root and five referenced lists into a fresh external cache.
- [ ] Bind each list's revision, bytes, SHA-256, and ordered case sequence to the root.
- [ ] Parse all 13 configurations in XML order, including repeated main-list configurations.
- [ ] Retain gles32-khr-glesext.txt and its configuration without a mandatory-semantics claim.
- [ ] Reject omission, duplication, reorder, source substitution, malformed XML, or classification.

## Verification

`make graphics-gles-full-closure-ledger-test` must test exact XML and list closure plus
hostile mutations. The receipt must report zero CTS executions and no qualification.
