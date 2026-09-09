# F06.1 — Define graphics module boundaries

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F06.1
Depends: F01
Evidence: [receipt](evidence.md)

Prerequisite lists: [F01](../../../01-contract/01-baseline/README.md).

## Outcome

The graphics implementation has explicit public ownership boundaries before feature code spreads.

## Starting points

- [graphics roadmap](../../../../README.md)
- [current baseline](../../../../baseline.md)
- [VirtIO-GPU 3D entry point](../../../../../../emulator/src/devices/virtio_gpu/three_d.rs)
- [browser 3D entry point](../../../../../../web/js/webgpu-3d.js)

## Checklist

- [x] Define distinct protocol, shared-runtime, shader frontend/backend, browser platform, guest,
  and test-fixture ownership roots with allowed dependency directions.
- [x] Map each existing bounded VirGL/Venus preparation seam to an owner without treating it as a
  completed Mesa, Venus, or performance layer.
- [x] Define a small public boundary per root and tests that may cross it; prohibit generated
  protocol output from silently becoming a hand-maintained public API.
- [x] Preserve `ExperimentalWebGpu3dRenderer` as a small facade while splitting its current
  dispatch/lifecycle responsibility into the defined browser-platform roots, with focused tests.
- [x] Record the reviewed boundary map and a search-based check that each proposed root exists or
  has an explicitly deferred creation task.

## Verification

- The map assigns every graphics roadmap phase to one ownership root and identifies no circular
  public dependency.
- A source search reproduces all current entry-point assignments; the facade's focused display,
  module-import, and asset-stamp checks pass after the boundary split.
