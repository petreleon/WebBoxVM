# F02.1 — Define the immutable graphics-input inventory

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F02.1
Depends: F01
Evidence: pending

Prerequisite lists: [F01](../../01-baseline/README.md).

## Outcome

One small, reviewable manifest names every authoritative input and immutable byte identity.

## Starting points

- [primary sources](../../../../sources.md)
- [VirGL2 boundary](../../../../../../research/virgl2-capset.md)
- [Venus foundations](../../../../../../research/venus-foundations.md)

## Checklist

- [ ] Define a maintained manifest schema with immutable URL, revision, SHA-256, source family,
  license, local cache location, and generated-code role for every entry.
- [ ] Add entries for Linux UAPI; Mesa/VirGL/Venus; virglrenderer/venus-protocol; GL/GLES,
  GLSL/ESSL, Vulkan, SPIR-V, WebGPU/WGSL; and selected independent reference suites.
- [ ] Reject mutable branch-only references and keep fetched bytes outside maintained source.
- [ ] Validate manifest structure and the complete required source-family inventory with a nonzero,
  implementation-independent test.

## Verification

- The manifest parser accepts the reviewed inventory and rejects a missing family, bad digest, or
  mutable source reference.
- F02.2 fetches its immutable bytes; this task does not treat planned downloads as evidence.
