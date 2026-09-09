# F02.3.3.4.4 — Close the WebGPU generator-input boundary

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.3.3.4.4
Depends: F02.1, F02.2, F02.3.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.1](../../../../01-input-inventory/README.md),
[F02.2](../../../../02-fetch-verifier/README.md), and
[F02.3.1](../../../01-provenance-record/README.md).

## Outcome

The local immutable inventory accepts only the reviewed `webgpu-idl` WebIDL
binding/interop input; every other identifier or identity mismatch is rejected. This is
metadata-provenance eligibility only, not a generator or graphics-runtime claim.

## Starting points

- [current immutable inventory](../../../../01-input-inventory/manifest.toml)
- [F02.3.1 validator](../../../01-provenance-record/README.md)
- [GPUWeb license at the reviewed pin](https://github.com/gpuweb/gpuweb/blob/e95743d3940e0ff3c267ab55ced9ae6120c7d416/LICENSE.md)
- [WebGPU generator-source audit](../../../../../../../../../research/webgpu-generator-source-audit.md)

## Checklist

- [x] [F02.3.3.4.4.1 — Admit a reviewed WebGPU WebIDL source](01-webidl-source-admission/README.md)
- [x] [F02.3.3.4.4.2 — Renew records after the WebIDL inventory change](02-lock-record-renewal/README.md)
- [x] [F02.3.3.4.4.3 — Bind a WebGPU WebIDL generator record](03-webidl-generator-record/README.md)
- [x] [F02.3.3.4.4.4 — Close the WebGPU generator-input boundary](04-boundary-closure/README.md)

## Verification

- Only the frozen `webgpu-idl` identity validates as the WebGPU WebIDL binding/interop input.
- Reference documents, WGSL/grammar inputs, CTS inputs, and renamed or role-only spoofs reject
  offline.

## Boundary probe

`webgpu_generator_boundary.py` validates the local F02.1 layout, including its lock, against the
sibling marker's frozen WebIDL identity. It neither fetches nor reads upstream WebIDL payloads.
Run the hermetic suite with `PYTHONDONTWRITEBYTECODE=1 python3 webgpu_generator_boundary_test.py`
from this folder.

## Split rationale

A separate source-definition audit identified an immutable GPUWeb WebIDL artifact that Dawn consumes
through a real generator. Admission changed the shared inventory and canonical lock, so existing
lock-bound records renewed before the metadata-only WebIDL fixture was bound. The final boundary
check remains separate so a source pin, record renewal, or provenance marker cannot be mistaken for
a browser API, guest renderer, or compatibility result.
