# F02.5.2.3 — Admit records and publish the no-claim receipt

[Parent task](../README.md) · [Worker instructions](../../../../../../workflow.md)

Task: F02.5.2.3
Depends: F02.5.1, F02.5.2.1, F02.5.2.2
Evidence: pending

## Outcome

The four Khronos source records and the bounded WebBoxVM Vulkan artifact form one validated catalog.
The receipt records a fresh source verification and rebuild while explicitly leaving support,
conformance, certification, profile support, and performance false.

## Starting points

- [normative root records](../01-normative-root-pins/normative_roots.py)
- [local definition task](../02-vulkan-local-definition/README.md)
- [streamed artifact verifier](../../01-authority-and-transform-boundary/source_role_artifacts.py)

## Checklist

- [ ] Bind the local artifact to exact input identities and canonical source-builder argv.
- [ ] Validate actual cached source and artifact bytes before consumer admission.
- [ ] Test false authority, source substitution, partial inputs, and all prohibited positive claims.
- [ ] Attach the audited receipt and complete the parent only after its checks pass.

## Verification

- Consumer admission requires both structural catalog validation and streamed artifact verification.
- The record does not authorize suite execution; F02.5.3 separately owns unmodified CTS roots.
