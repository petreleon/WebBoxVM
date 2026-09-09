# F02.3.3.4.4 — Hold the WebGPU generator-input boundary

[Parent task](../README.md) · [Worker instructions](../../../../../../../workflow.md)

Task: F02.3.3.4.4
Depends: F02.1, F02.2, F02.3.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.1](../../../../01-input-inventory/README.md),
[F02.2](../../../../02-fetch-verifier/README.md), and
[F02.3.1](../../../01-provenance-record/README.md).

## Outcome

WebGPU reference documents remain ineligible as generator inputs until a reviewed, immutable,
machine-readable WebGPU input is available; no fixture or runtime claim fills that gap.

## Starting points

- [current immutable inventory](../../../../01-input-inventory/manifest.toml)
- [F02.3.1 validator](../../../01-provenance-record/README.md)
- [GPUWeb license at the current pin](https://github.com/gpuweb/gpuweb/blob/e0aff163a37eb3633ffd612e2a943ceb6196d6af/LICENSE.md)
- [WebGPU generator-source audit](../../../../../../../../../research/webgpu-generator-source-audit.md)

## Checklist

- [x] [F02.3.3.4.4.1 — Admit a reviewed WebGPU WebIDL source](01-webidl-source-admission/README.md)
- [x] [F02.3.3.4.4.2 — Renew records after the WebIDL inventory change](02-lock-record-renewal/README.md)
- [x] [F02.3.3.4.4.3 — Bind a WebGPU WebIDL generator record](03-webidl-generator-record/README.md)
- [ ] [F02.3.3.4.4.4 — Close the WebGPU generator-input boundary](04-boundary-closure/README.md)

## Verification

- A reference-only or WGSL-only input cannot validate as a WebGPU generator input offline.
- This child stays open until the immutable inventory contains an honest WebGPU generator input.

## Blocker probe

`webgpu_generator_boundary.py` reads only the local F02.1 manifest. It accepts a blocked result
only while no input has the exact `future WebGPU generator input` designation, and it rejects every
WGSL-derived input—including the accepted grammar—as a WebGPU generator input. Run the hermetic
suite with `PYTHONDONTWRITEBYTECODE=1 python3 webgpu_generator_boundary_test.py` from this folder.

## Split rationale

The original blocked boundary became actionable only after a separate source-definition audit found
an immutable GPUWeb WebIDL artifact that Dawn consumes through a real generator. Admission changes
the shared inventory and its canonical lock; all existing lock-bound records must then renew before
a new WebIDL fixture can be honest. The final boundary check is intentionally separate so a source
pin, record renewal, or provenance marker cannot be mistaken for a browser API, guest renderer, or
compatibility result.
