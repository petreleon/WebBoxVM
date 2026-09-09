# F02.3.3.4.4.4 — Close the WebGPU generator-input boundary

[Parent task](../README.md) · [Worker instructions](../../../../../../../../workflow.md)

Task: F02.3.3.4.4.4
Depends: F02.3.3.4.4.1, F02.3.3.4.4.2, F02.3.3.4.4.3
Evidence: pending

Prerequisite lists: [WebIDL source admission](../01-webidl-source-admission/README.md),
[renewed records](../02-lock-record-renewal/README.md), and the
[WebIDL provenance marker](../03-webidl-generator-record/README.md).

## Outcome

The local boundary proof accepts exactly the reviewed WebGPU WebIDL generator
input and rejects semantic, WGSL, grammar, native-C-schema, and CTS substitutes;
the completed foundation records no graphics runtime claim.

## Starting points

- [current boundary probe](../webgpu_generator_boundary.py)
- [current boundary tests](../webgpu_generator_boundary_test.py)
- [source-definition audit](../../../../../../../../../../research/webgpu-generator-source-audit.md)
- [WebIDL marker task](../03-webidl-generator-record/README.md)

## Checklist

- [ ] Make the boundary probe accept only the exact reviewed `webgpu-idl` role and family.
- [ ] Add offline positive and hostile-input cases for all prohibited substitute sources.
- [ ] Replace the blocked receipt with a complete local foundation receipt and update parent status.
- [ ] Re-run the focused suites, source limits, full local suite, roadmap checker, and whitespace check.

## Verification

- The WebIDL input passes only when its family and explicit generator role are intact.
- Reference documents and non-WebIDL sources cannot pass by renaming or role changes alone.

## Scope limit

Closing this provenance boundary does not implement a WebGPU API, invoke browser WebGPU, expose
guest graphics, translate VirGL or Venus, render a frame, certify a profile, or measure performance.
