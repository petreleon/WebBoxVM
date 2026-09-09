# R04 — Implement texture subresource layout and transfers

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: R04
Depends: R02, R03
Evidence: pending

Prerequisite lists: [R02](../02-object-identity/README.md), [R03](../03-buffer-transfers/README.md).

## Outcome

Mip levels, layers and row/slice strides have one checked representation shared by both APIs.

## Starting points

- [emulator/src/devices/virtio_gpu/three_d/virgl/resource.rs](../../../../../emulator/src/devices/virtio_gpu/three_d/virgl/resource.rs)
- [emulator/src/devices/virtio_gpu/resource_transfer/readback.rs](../../../../../emulator/src/devices/virtio_gpu/resource_transfer/readback.rs)
- [web/js/webgpu-virgl-texture-upload.js](../../../../../web/js/webgpu-virgl-texture-upload.js)

## Checklist

- [ ] Define format/block geometry and subresource extents with checked arithmetic.
- [ ] Implement mip/layer/3D-region staging and row padding; split format families into child tasks.
- [ ] Preserve untouched texels and reject illegal sample, stride, aspect and dimension
  combinations.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Sentinel-texture round trips cover nonzero mip/layer, odd dimensions and padded rows.
- Reference images and byte counts match for each advertised format; unsupported formats fail
  explicitly.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
