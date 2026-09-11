# F02.4.4.1.5.4.4.5.2.2.3.1 — Bind the sealed GLES capture

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.4.4.5.2.2.3.1
Depends: F02.4.4.1.5.4.4.5.2.2.2
Evidence: pending

Prerequisite lists: the [capture receipt](../../02-capture-and-replay/evidence.md), the
[closure contract](../../01-closure-contract/README.md), and the [successor integration](../../../01-successor-integration/README.md).

## Outcome

Create one cache-independent, self-checked binding for the sealed six-member GLES capture. It must
name the capture marker and the exact contract/closure/configuration identities without treating a
captured-unadmitted marker as active cache freshness, source admission, or an F03 input.

## Starting points

- [GLES capture/replay implementation](../../02-capture-and-replay/gles_capture.py)
- [capture marker grammar](../../02-capture-and-replay/gles_capture_marker.py)
- [multi-suite integration](../../../01-successor-integration/multi_suite_successor_integration.py)

## Checklist

- [ ] Bind the exact marker body and its six ordered source identities to the frozen closure contract.
- [ ] Keep the F02 cache grammar separate from the successor-only marker namespace.
- [ ] Require a complete offline replay before a live external cache can satisfy the binding.
- [ ] Reject marker substitution, stale contract/configuration identity, partial member lists, and claimed readiness.
- [ ] Add bounded positive and hostile tests without modifying active inventory, F03, or support state.

## Verification

- A durable binding describes a verified capture; it does not preserve raw source payloads in Git.
- The next child alone evaluates active F02/F03 consumers; no transition is allowed here.
