# F02.3.2 — Bind ABI fixtures and adapters

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.3.2
Depends: F02.1, F02.2, F02.3.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.3.1](../01-provenance-record/README.md) and
[F02.2](../../02-fetch-verifier/README.md).

## Outcome

ABI-adjacent fixtures and maintained adapters declare their reviewed upstream inputs and honest origin.

## Starting points

- [Linux UAPI input](../../01-input-inventory/manifest.toml)
- [guest VirGL header](../../../../../../../guest/virgl-clear-demo/uapi.h)
- [guest WebGPU UAPI](../../../../../../../guest/webgpu-demo/uapi.h)
- [VirtIO GPU capset](../../../../../../../emulator/src/devices/virtio_gpu/three_d/capset.rs)

## Checklist

- [x] Inventory current ABI-facing headers, fixtures, and adapters that need a provenance record.
- [x] Bind each record to exact verified Linux, VirGL, virglrenderer, or Venus inputs as applicable.
- [x] Mark copied upstream bytes separately from maintained hand-written protocol adapters.
- [x] Add a focused check that every selected ABI record resolves through F02.3.1's validator.

## Verification

- Each selected ABI artifact has an output hash and only declared input IDs/digests.
- Changing its declared input or artifact origin fails validation without changing runtime behavior.
